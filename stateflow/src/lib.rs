//! A simple state machine library for Rust.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// Represents an action with a type and command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The type of the action.
    pub action_type: String,
    /// The command to execute.
    pub command: String,
}

/// A struct representing a state and its transitions, including actions on enter and exit.
#[derive(Debug)]
struct State {
    name: String,
    on_enter_actions: Vec<Action>,
    on_exit_actions: Vec<Action>,
    transitions: HashMap<String, Transition>, // Key: event name, Value: Transition instance
    validations: Vec<ValidationRule>,         // State validation rules
}

/// Represents a transition between states, including actions and validations.
#[derive(Debug)]
struct Transition {
    to_state: String,
    actions: Vec<Action>,
    validations: Vec<ValidationRule>, // Transition validation rules
}

/// Represents a validation rule applied to the memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationRule {
    field: String,
    rules: Vec<FieldRule>,
    condition: Option<Condition>, // Optional condition for conditional validations
}

/// Represents a single rule for a field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum FieldRule {
    #[serde(rename = "type_check")]
    TypeCheck { expected_type: String },
    #[serde(rename = "nullable")]
    Nullable { is_nullable: bool },
    #[serde(rename = "min_value")]
    MinValue { value: f64 },
    #[serde(rename = "max_value")]
    MaxValue { value: f64 },
    #[serde(rename = "editable")]
    Editable { is_editable: bool },
    #[serde(rename = "read_only")]
    ReadOnly { is_read_only: bool },
    #[serde(rename = "enum")]
    Enum { values: Vec<Value> },
    // Add more rules as needed
}

/// Represents a condition for conditional validations.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Condition {
    field: String,
    operator: String,
    value: Value,
}

/// Represents the configuration of a state machine loaded from JSON.
#[derive(Debug, Serialize, Deserialize)]
struct StateMachineConfig {
    states: Vec<StateConfig>,
    transitions: Vec<TransitionConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StateConfig {
    name: String,
    #[serde(default)]
    on_enter_actions: Vec<ActionConfig>,
    #[serde(default)]
    on_exit_actions: Vec<ActionConfig>,
    validations: Option<Vec<ValidationRule>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransitionConfig {
    from: String,
    event: String,
    to: String,
    #[serde(default)]
    actions: Vec<ActionConfig>, // Actions triggered during the transition
    validations: Option<Vec<ValidationRule>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActionConfig {
    action_type: String,
    command: String,
}

type ActionHandler<C> = dyn Fn(&Action, &mut Map<String, Value>, &mut C) + Send + Sync;

/// The state machine containing all states, the current state, memory, context, and handlers.
pub struct StateMachine<C> {
    states: Arc<RwLock<HashMap<String, State>>>, // Key: state name, Value: state instance
    current_state: Arc<RwLock<String>>,
    action_handler: Arc<ActionHandler<C>>, // Callback function for handling actions
    /// The memory of the state machine. Memory is a map of key-value pairs
    pub memory: Arc<RwLock<Map<String, Value>>>,
    /// The user-provided context
    pub context: Arc<RwLock<C>>,
}

impl<C> StateMachine<C> {
    /// Creates a new state machine from a configuration string, optionally restoring to a specific state.
    /// `initial_state` can be used to restore the machine to a saved state.
    /// `memory` is the mutable state or memory of the state machine.
    pub fn new<F>(
        config_content: &str,
        initial_state: Option<String>,
        action_handler: F,
        memory: Map<String, Value>,
        context: C,
    ) -> Result<Self, String>
    where
        F: Fn(&Action, &mut Map<String, Value>, &mut C) + Send + Sync + 'static,
    {
        // Generate and compile the JSON schema
        let schema = Self::generate_and_compile_schema()?;

        // Parse the configuration from the provided string
        let config_value: serde_json::Value = serde_json::from_str(config_content)
            .map_err(|err| format!("Invalid JSON format in configuration: {}", err))?;

        // Validate the configuration against the schema
        let compiled_schema = jsonschema::Validator::new(&schema)
            .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;
        if let Err(errors) = compiled_schema.validate(&config_value) {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!(
                "JSON configuration does not conform to schema: {}",
                error_messages.join(", ")
            ));
        }

        // Deserialize the configuration
        let config: StateMachineConfig = serde_json::from_value(config_value)
            .map_err(|err| format!("Failed to deserialize configuration: {}", err))?;

        // Validate the config
        Self::validate_config(&config)?;

        // Create states and populate transitions
        let mut states = HashMap::new();
        for state_config in &config.states {
            let state = State {
                name: state_config.name.clone(),
                on_enter_actions: Self::create_actions(&state_config.on_enter_actions),
                on_exit_actions: Self::create_actions(&state_config.on_exit_actions),
                transitions: HashMap::new(),
                validations: state_config.validations.clone().unwrap_or_default(),
            };
            states.insert(state_config.name.clone(), state);
        }

        // Populate transitions for each state
        for transition_config in &config.transitions {
            if let Some(state) = states.get_mut(&transition_config.from) {
                let transition = Transition {
                    to_state: transition_config.to.clone(),
                    actions: Self::create_actions(&transition_config.actions),
                    validations: transition_config.validations.clone().unwrap_or_default(),
                };
                state
                    .transitions
                    .insert(transition_config.event.clone(), transition);
            }
        }

        // Determine the starting state: use provided initial state or default to the first state
        let current_state = initial_state.unwrap_or_else(|| config.states[0].name.clone());

        Ok(StateMachine {
            states: Arc::new(RwLock::new(states)),
            current_state: Arc::new(RwLock::new(current_state)),
            action_handler: Arc::new(action_handler),
            memory: Arc::new(RwLock::new(memory)),
            context: Arc::new(RwLock::new(context)),
        })
    }

    /// Generates and compiles the JSON schema for the state machine configuration.
    fn generate_and_compile_schema() -> Result<serde_json::Value, String> {
        // Define the JSON schema as a serde_json::Value
        let schema_json = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "StateMachineConfig",
            "type": "object",
            "required": ["states", "transitions"],
            "properties": {
                "states": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": { "type": "string" },
                            "on_enter_actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "on_exit_actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "validations": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/validation_rule" }
                            }
                        }
                    }
                },
                "transitions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from", "event", "to"],
                        "properties": {
                            "from": { "type": "string" },
                            "event": { "type": "string" },
                            "to": { "type": "string" },
                            "actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "validations": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/validation_rule" }
                            }
                        }
                    }
                }
            },
            "definitions": {
                "action": {
                    "type": "object",
                    "required": ["action_type", "command"],
                    "properties": {
                        "action_type": { "type": "string" },
                        "command": { "type": "string" }
                    }
                },
                "validation_rule": {
                    "type": "object",
                    "required": ["field", "rules"],
                    "properties": {
                        "field": { "type": "string" },
                        "rules": {
                            "type": "array",
                            "items": { "$ref": "#/definitions/field_rule" }
                        },
                        "condition": { "$ref": "#/definitions/condition" }
                    }
                },
                "field_rule": {
                    "type": "object",
                    "oneOf": [
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "type_check" },
                                "expected_type": { "type": "string" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "nullable" },
                                "is_nullable": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "min_value" },
                                "value": { "type": "number" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "max_value" },
                                "value": { "type": "number" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "editable" },
                                "is_editable": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "read_only" },
                                "is_read_only": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "enum" },
                                "values": {
                                    "type": "array",
                                    "items": {}
                                }
                            }
                        }
                        // Add more field rule schemas as needed
                    ]
                },
                "condition": {
                    "type": "object",
                    "required": ["field", "operator", "value"],
                    "properties": {
                        "field": { "type": "string" },
                        "operator": { "type": "string" },
                        "value": {}
                    }
                }
            }
        });

        Ok(schema_json)
    }

    /// Creates actions from the action configuration.
    fn create_actions(action_configs: &[ActionConfig]) -> Vec<Action> {
        action_configs
            .iter()
            .map(|config| Action {
                action_type: config.action_type.clone(),
                command: config.command.clone(),
            })
            .collect()
    }

    /// Validates the state machine configuration.
    fn validate_config(config: &StateMachineConfig) -> Result<(), String> {
        if config.states.is_empty() {
            return Err("State machine must have at least one state.".into());
        }

        let mut state_set = std::collections::HashSet::new();
        for state in &config.states {
            if !state_set.insert(&state.name) {
                return Err(format!("Duplicate state found: {}", state.name));
            }
        }

        for transition in &config.transitions {
            if !config.states.iter().any(|s| s.name == transition.from) {
                return Err(format!(
                    "Transition 'from' state '{}' is not defined in the states list.",
                    transition.from
                ));
            }
            if !config.states.iter().any(|s| s.name == transition.to) {
                return Err(format!(
                    "Transition 'to' state '{}' is not defined in the states list.",
                    transition.to
                ));
            }
            if transition.event.trim().is_empty() {
                return Err(format!(
                    "Transition from '{}' to '{}' has an empty event.",
                    transition.from, transition.to
                ));
            }
        }

        Ok(())
    }

    /// Triggers an event, causing a state transition if applicable and executing actions.
    pub fn trigger(&self, event: &str) -> Result<(), String> {
        let current_state_name = self.current_state.read().unwrap().clone();
        let states = self.states.read().unwrap();

        if let Some(current_state) = states.get(&current_state_name) {
            // Get mutable references to the memory and context
            let mut memory = self.memory.write().unwrap();
            let mut context = self.context.write().unwrap();

            // Execute state validations
            Self::evaluate_validations(&current_state.validations, &memory)?;

            if let Some(transition) = current_state.transitions.get(event) {
                // Execute transition validations
                Self::evaluate_validations(&transition.validations, &memory)?;

                // Execute on-exit actions
                self.execute_actions(&current_state.on_exit_actions, &mut memory, &mut context);

                // Execute transition actions
                self.execute_actions(&transition.actions, &mut memory, &mut context);

                // Set the new current state
                *self.current_state.write().unwrap() = transition.to_state.clone();

                if let Some(next_state) = states.get(&transition.to_state) {
                    // Execute on-enter actions of the next state
                    self.execute_actions(&next_state.on_enter_actions, &mut memory, &mut context);
                }

                log::trace!(
                    "Transitioning from {} to {} on event '{}'",
                    current_state_name,
                    transition.to_state,
                    event
                );
                return Ok(());
            }
        }
        Err(format!(
            "No transition found for event '{}' from state '{}'.",
            event, current_state_name
        ))
    }

    /// Executes a list of actions using the provided action handler.
    fn execute_actions(
        &self,
        actions: &[Action],
        memory: &mut Map<String, Value>,
        context: &mut C,
    ) {
        for action in actions {
            (self.action_handler)(action, memory, context); // Call the user-provided callback with the action, memory, and context
        }
    }

    /// Evaluates a list of validation rules against the memory.
    fn evaluate_validations(
        validations: &[ValidationRule],
        memory: &Map<String, Value>,
    ) -> Result<(), String> {
        for validation in validations {
            // Check condition if present
            if let Some(condition) = &validation.condition {
                if !Self::evaluate_condition(condition, memory)? {
                    // Condition not met, skip validation
                    continue;
                }
            }

            // Get the value from the memory
            let field_value = memory.get(&validation.field);

            for rule in &validation.rules {
                match rule {
                    FieldRule::TypeCheck { expected_type } => {
                        if let Some(value) = field_value {
                            let actual_type = Self::get_type_name(value);
                            if actual_type != expected_type {
                                return Err(format!(
                                    "Validation failed: Field '{}' expected type '{}', got '{}'",
                                    validation.field, expected_type, actual_type
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is missing in memory",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::Nullable { is_nullable } => {
                        if !*is_nullable && field_value.is_none() {
                            return Err(format!(
                                "Validation failed: Field '{}' cannot be null",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::MinValue { value: min_value } => {
                        if let Some(Value::Number(num)) = field_value {
                            if num.as_f64().unwrap_or(f64::NAN) < *min_value {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is less than minimum '{}'",
                                    validation.field, num, min_value
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is not a number",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::MaxValue { value: max_value } => {
                        if let Some(Value::Number(num)) = field_value {
                            if num.as_f64().unwrap_or(f64::NAN) > *max_value {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is greater than maximum '{}'",
                                    validation.field, num, max_value
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is not a number",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::Editable { is_editable: _ }
                    | FieldRule::ReadOnly { is_read_only: _ } => {
                        // Not implemented
                    }
                    FieldRule::Enum { values } => {
                        if let Some(value) = field_value {
                            if !values.contains(value) {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is not in enum {:?}",
                                    validation.field, value, values
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is missing in memory",
                                validation.field
                            ));
                        }
                    } // Handle more rules as needed
                }
            }
        }
        Ok(())
    }

    /// Evaluates a condition against the memory.
    fn evaluate_condition(
        condition: &Condition,
        memory: &Map<String, Value>,
    ) -> Result<bool, String> {
        let field_value = memory.get(&condition.field);
        if let Some(actual_value) = field_value {
            let result = match condition.operator.as_str() {
                "==" => actual_value == &condition.value,
                "!=" => actual_value != &condition.value,
                ">" => Self::compare_values(
                    actual_value,
                    &condition.value,
                    std::cmp::Ordering::Greater,
                )?,
                "<" => {
                    Self::compare_values(actual_value, &condition.value, std::cmp::Ordering::Less)?
                }
                ">=" => {
                    let ordering = Self::compare_values_ordering(actual_value, &condition.value)?;
                    ordering == std::cmp::Ordering::Greater || ordering == std::cmp::Ordering::Equal
                }
                "<=" => {
                    let ordering = Self::compare_values_ordering(actual_value, &condition.value)?;
                    ordering == std::cmp::Ordering::Less || ordering == std::cmp::Ordering::Equal
                }
                _ => return Err(format!("Unsupported operator '{}'", condition.operator)),
            };
            Ok(result)
        } else {
            Err(format!(
                "Condition evaluation failed: Field '{}' is missing in memory",
                condition.field
            ))
        }
    }

    /// Compares two serde_json::Value numbers based on the expected ordering.
    fn compare_values(
        actual: &Value,
        expected: &Value,
        ordering: std::cmp::Ordering,
    ) -> Result<bool, String> {
        let actual_num = actual
            .as_f64()
            .ok_or_else(|| format!("Cannot compare non-numeric value '{}' in condition", actual))?;
        let expected_num = expected.as_f64().ok_or_else(|| {
            format!(
                "Cannot compare non-numeric value '{}' in condition",
                expected
            )
        })?;
        Ok(actual_num.partial_cmp(&expected_num) == Some(ordering))
    }

    /// Compares two serde_json::Value numbers and returns the ordering.
    fn compare_values_ordering(
        actual: &Value,
        expected: &Value,
    ) -> Result<std::cmp::Ordering, String> {
        let actual_num = actual
            .as_f64()
            .ok_or_else(|| format!("Cannot compare non-numeric value '{}' in condition", actual))?;
        let expected_num = expected.as_f64().ok_or_else(|| {
            format!(
                "Cannot compare non-numeric value '{}' in condition",
                expected
            )
        })?;
        Ok(actual_num
            .partial_cmp(&expected_num)
            .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Returns a string representing the type of the serde_json::Value.
    fn get_type_name(value: &Value) -> &str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Returns the current state of the state machine.
    pub fn get_current_state(&self) -> Result<String, String> {
        let current_state = self.current_state.read().unwrap();
        Ok((*current_state).clone())
    }
}

/// Implementing the Display trait to render the state machine as a string.
impl<C> Display for StateMachine<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let states = self.states.read().unwrap();
        let current_state = self.current_state.read().unwrap();

        writeln!(f, "State Machine Diagram:")?;
        writeln!(f, "======================")?;

        for (state_name, state) in &*states {
            let marker = if *state_name == *current_state {
                "->" // Indicate the current state
            } else {
                "  "
            };
            writeln!(f, "{} State: {}", marker, state.name)?;

            for (event, transition) in &state.transitions {
                writeln!(f, "      -[{}]-> {}", event, transition.to_state)?;
            }
        }

        writeln!(f, "======================")
    }
}
type AsyncActionHandler<C> = Arc<
    dyn Fn(Action, &mut Map<String, Value>, &mut C) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// The async state machine containing all states, the current state, memory, context, and handlers.
pub struct AsyncStateMachine<C> {
    states: Arc<RwLock<HashMap<String, State>>>,
    current_state: Arc<RwLock<String>>,
    action_handler: AsyncActionHandler<C>,
    /// The memory of the state machine. Memory is a map of key-value pairs
    pub memory: Arc<RwLock<Map<String, Value>>>,
    /// The user-provided context
    pub context: Arc<RwLock<C>>,
}

impl<C: Send + Sync + 'static> AsyncStateMachine<C> {
    /// Creates a new async state machine from a configuration string.
    pub fn new<F, Fut>(
        config_content: &str,
        initial_state: Option<String>,
        action_handler: F,
        memory: Map<String, Value>,
        context: C,
    ) -> Result<Self, String>
    where
        F: Fn(Action, &mut Map<String, Value>, &mut C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // Reuse existing validation and initialization logic
        let schema = Self::generate_and_compile_schema()?;

        // Parse and validate configuration
        let config_value: serde_json::Value = serde_json::from_str(config_content)
            .map_err(|err| format!("Invalid JSON format in configuration: {}", err))?;

        let compiled_schema = jsonschema::Validator::new(&schema)
            .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;

        if let Err(errors) = compiled_schema.validate(&config_value) {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!(
                "JSON configuration does not conform to schema: {}",
                error_messages.join(", ")
            ));
        }

        let config: StateMachineConfig = serde_json::from_value(config_value)
            .map_err(|err| format!("Failed to deserialize configuration: {}", err))?;

        Self::validate_config(&config)?;

        // Create states with transitions
        let mut states = HashMap::new();
        for state_config in &config.states {
            let state = State {
                name: state_config.name.clone(),
                on_enter_actions: Self::create_actions(&state_config.on_enter_actions),
                on_exit_actions: Self::create_actions(&state_config.on_exit_actions),
                transitions: HashMap::new(),
                validations: state_config.validations.clone().unwrap_or_default(),
            };
            states.insert(state_config.name.clone(), state);
        }

        // Populate transitions
        for transition_config in &config.transitions {
            if let Some(state) = states.get_mut(&transition_config.from) {
                let transition = Transition {
                    to_state: transition_config.to.clone(),
                    actions: Self::create_actions(&transition_config.actions),
                    validations: transition_config.validations.clone().unwrap_or_default(),
                };
                state
                    .transitions
                    .insert(transition_config.event.clone(), transition);
            }
        }

        // Wrap the action handler to return a boxed future
        let wrapped_handler = Arc::new(
            move |action: Action, memory: &mut Map<String, Value>, context: &mut C| {
                Box::pin(action_handler(action, memory, context))
                    as Pin<Box<dyn Future<Output = ()> + Send>>
            },
        );

        let current_state = initial_state.unwrap_or_else(|| config.states[0].name.clone());

        Ok(AsyncStateMachine {
            states: Arc::new(RwLock::new(states)),
            current_state: Arc::new(RwLock::new(current_state)),
            action_handler: wrapped_handler,
            memory: Arc::new(RwLock::new(memory)),
            context: Arc::new(RwLock::new(context)),
        })
    }

    /// Asynchronously triggers an event, causing a state transition if applicable and executing actions.
    pub async fn trigger(&self, event: &str) -> Result<(), String> {
        let current_state_name = self.current_state.read().unwrap().clone();
        let states = self.states.read().unwrap();

        if let Some(current_state) = states.get(&current_state_name) {
            let mut memory = self.memory.write().unwrap();
            let mut context = self.context.write().unwrap();

            // Execute state validations
            Self::evaluate_validations(&current_state.validations, &memory)?;

            if let Some(transition) = current_state.transitions.get(event) {
                // Execute transition validations
                Self::evaluate_validations(&transition.validations, &memory)?;

                // Execute on-exit actions
                self.execute_actions(&current_state.on_exit_actions, &mut memory, &mut context)
                    .await;

                // Execute transition actions
                self.execute_actions(&transition.actions, &mut memory, &mut context)
                    .await;

                // Set the new current state
                *self.current_state.write().unwrap() = transition.to_state.clone();

                if let Some(next_state) = states.get(&transition.to_state) {
                    // Execute on-enter actions of the next state
                    self.execute_actions(&next_state.on_enter_actions, &mut memory, &mut context)
                        .await;
                }

                log::trace!(
                    "Transitioning from {} to {} on event '{}'",
                    current_state_name,
                    transition.to_state,
                    event
                );
                return Ok(());
            }
        }
        Err(format!(
            "No transition found for event '{}' from state '{}'.",
            event, current_state_name
        ))
    }

    /// Asynchronously executes a list of actions using the provided action handler.
    async fn execute_actions(
        &self,
        actions: &[Action],
        memory: &mut Map<String, Value>,
        context: &mut C,
    ) {
        for action in actions {
            (self.action_handler)(action.clone(), memory, context).await;
        }
    }

    /// Generates and compiles the JSON schema for the state machine configuration.
    fn generate_and_compile_schema() -> Result<serde_json::Value, String> {
        // Define the JSON schema as a serde_json::Value
        let schema_json = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "StateMachineConfig",
            "type": "object",
            "required": ["states", "transitions"],
            "properties": {
                "states": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": { "type": "string" },
                            "on_enter_actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "on_exit_actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "validations": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/validation_rule" }
                            }
                        }
                    }
                },
                "transitions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from", "event", "to"],
                        "properties": {
                            "from": { "type": "string" },
                            "event": { "type": "string" },
                            "to": { "type": "string" },
                            "actions": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/action" },
                                "default": []
                            },
                            "validations": {
                                "type": "array",
                                "items": { "$ref": "#/definitions/validation_rule" }
                            }
                        }
                    }
                }
            },
            "definitions": {
                "action": {
                    "type": "object",
                    "required": ["action_type", "command"],
                    "properties": {
                        "action_type": { "type": "string" },
                        "command": { "type": "string" }
                    }
                },
                "validation_rule": {
                    "type": "object",
                    "required": ["field", "rules"],
                    "properties": {
                        "field": { "type": "string" },
                        "rules": {
                            "type": "array",
                            "items": { "$ref": "#/definitions/field_rule" }
                        },
                        "condition": { "$ref": "#/definitions/condition" }
                    }
                },
                "field_rule": {
                    "type": "object",
                    "oneOf": [
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "type_check" },
                                "expected_type": { "type": "string" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "nullable" },
                                "is_nullable": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "min_value" },
                                "value": { "type": "number" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "max_value" },
                                "value": { "type": "number" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "editable" },
                                "is_editable": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "read_only" },
                                "is_read_only": { "type": "boolean" }
                            }
                        },
                        {
                            "type": "object",
                            "required": ["type"],
                            "properties": {
                                "type": { "const": "enum" },
                                "values": {
                                    "type": "array",
                                    "items": {}
                                }
                            }
                        }
                        // Add more field rule schemas as needed
                    ]
                },
                "condition": {
                    "type": "object",
                    "required": ["field", "operator", "value"],
                    "properties": {
                        "field": { "type": "string" },
                        "operator": { "type": "string" },
                        "value": {}
                    }
                }
            }
        });

        Ok(schema_json)
    }

    fn validate_config(config: &StateMachineConfig) -> Result<(), String> {
        if config.states.is_empty() {
            return Err("State machine must have at least one state.".into());
        }

        let mut state_set = std::collections::HashSet::new();
        for state in &config.states {
            if !state_set.insert(&state.name) {
                return Err(format!("Duplicate state found: {}", state.name));
            }
        }

        for transition in &config.transitions {
            if !config.states.iter().any(|s| s.name == transition.from) {
                return Err(format!(
                    "Transition 'from' state '{}' is not defined in the states list.",
                    transition.from
                ));
            }
            if !config.states.iter().any(|s| s.name == transition.to) {
                return Err(format!(
                    "Transition 'to' state '{}' is not defined in the states list.",
                    transition.to
                ));
            }
            if transition.event.trim().is_empty() {
                return Err(format!(
                    "Transition from '{}' to '{}' has an empty event.",
                    transition.from, transition.to
                ));
            }
        }

        Ok(())
    }

    /// Creates actions from the action configuration.
    fn create_actions(action_configs: &[ActionConfig]) -> Vec<Action> {
        action_configs
            .iter()
            .map(|config| Action {
                action_type: config.action_type.clone(),
                command: config.command.clone(),
            })
            .collect()
    }

    /// Evaluates a list of validation rules against the memory.
    fn evaluate_validations(
        validations: &[ValidationRule],
        memory: &Map<String, Value>,
    ) -> Result<(), String> {
        for validation in validations {
            // Check condition if present
            if let Some(condition) = &validation.condition {
                if !Self::evaluate_condition(condition, memory)? {
                    // Condition not met, skip validation
                    continue;
                }
            }

            // Get the value from the memory
            let field_value = memory.get(&validation.field);

            for rule in &validation.rules {
                match rule {
                    FieldRule::TypeCheck { expected_type } => {
                        if let Some(value) = field_value {
                            let actual_type = Self::get_type_name(value);
                            if actual_type != expected_type {
                                return Err(format!(
                                    "Validation failed: Field '{}' expected type '{}', got '{}'",
                                    validation.field, expected_type, actual_type
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is missing in memory",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::Nullable { is_nullable } => {
                        if !*is_nullable && field_value.is_none() {
                            return Err(format!(
                                "Validation failed: Field '{}' cannot be null",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::MinValue { value: min_value } => {
                        if let Some(Value::Number(num)) = field_value {
                            if num.as_f64().unwrap_or(f64::NAN) < *min_value {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is less than minimum '{}'",
                                    validation.field, num, min_value
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is not a number",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::MaxValue { value: max_value } => {
                        if let Some(Value::Number(num)) = field_value {
                            if num.as_f64().unwrap_or(f64::NAN) > *max_value {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is greater than maximum '{}'",
                                    validation.field, num, max_value
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is not a number",
                                validation.field
                            ));
                        }
                    }
                    FieldRule::Editable { is_editable: _ }
                    | FieldRule::ReadOnly { is_read_only: _ } => {
                        // Not implemented
                    }
                    FieldRule::Enum { values } => {
                        if let Some(value) = field_value {
                            if !values.contains(value) {
                                return Err(format!(
                                    "Validation failed: Field '{}' value '{}' is not in enum {:?}",
                                    validation.field, value, values
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Validation failed: Field '{}' is missing in memory",
                                validation.field
                            ));
                        }
                    } // Handle more rules as needed
                }
            }
        }
        Ok(())
    }

    /// Evaluates a condition against the memory.
    fn evaluate_condition(
        condition: &Condition,
        memory: &Map<String, Value>,
    ) -> Result<bool, String> {
        let field_value = memory.get(&condition.field);
        if let Some(actual_value) = field_value {
            let result = match condition.operator.as_str() {
                "==" => actual_value == &condition.value,
                "!=" => actual_value != &condition.value,
                ">" => Self::compare_values(
                    actual_value,
                    &condition.value,
                    std::cmp::Ordering::Greater,
                )?,
                "<" => {
                    Self::compare_values(actual_value, &condition.value, std::cmp::Ordering::Less)?
                }
                ">=" => {
                    let ordering = Self::compare_values_ordering(actual_value, &condition.value)?;
                    ordering == std::cmp::Ordering::Greater || ordering == std::cmp::Ordering::Equal
                }
                "<=" => {
                    let ordering = Self::compare_values_ordering(actual_value, &condition.value)?;
                    ordering == std::cmp::Ordering::Less || ordering == std::cmp::Ordering::Equal
                }
                _ => return Err(format!("Unsupported operator '{}'", condition.operator)),
            };
            Ok(result)
        } else {
            Err(format!(
                "Condition evaluation failed: Field '{}' is missing in memory",
                condition.field
            ))
        }
    }

    /// Compares two serde_json::Value numbers and returns the ordering.
    fn compare_values_ordering(
        actual: &Value,
        expected: &Value,
    ) -> Result<std::cmp::Ordering, String> {
        let actual_num = actual
            .as_f64()
            .ok_or_else(|| format!("Cannot compare non-numeric value '{}' in condition", actual))?;
        let expected_num = expected.as_f64().ok_or_else(|| {
            format!(
                "Cannot compare non-numeric value '{}' in condition",
                expected
            )
        })?;
        Ok(actual_num
            .partial_cmp(&expected_num)
            .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Returns a string representing the type of the serde_json::Value.
    fn get_type_name(value: &Value) -> &str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Returns the current state of the state machine.
    pub fn get_current_state(&self) -> Result<String, String> {
        let current_state = self.current_state.read().unwrap();
        Ok((*current_state).clone())
    }

    /// Compares two serde_json::Value numbers based on the expected ordering.
    fn compare_values(
        actual: &Value,
        expected: &Value,
        ordering: std::cmp::Ordering,
    ) -> Result<bool, String> {
        let actual_num = actual
            .as_f64()
            .ok_or_else(|| format!("Cannot compare non-numeric value '{}' in condition", actual))?;
        let expected_num = expected.as_f64().ok_or_else(|| {
            format!(
                "Cannot compare non-numeric value '{}' in condition",
                expected
            )
        })?;
        Ok(actual_num.partial_cmp(&expected_num) == Some(ordering))
    }
}

/// Implementing the Display trait to render the state machine as a string.
impl<C> Display for AsyncStateMachine<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let states = self.states.read().unwrap();
        let current_state = self.current_state.read().unwrap();

        writeln!(f, "Async State Machine Diagram:")?;
        writeln!(f, "===========================")?;

        for (state_name, state) in &*states {
            let marker = if *state_name == *current_state {
                "->" // Indicate the current state
            } else {
                "  "
            };
            writeln!(f, "{} State: {}", marker, state.name)?;

            for (event, transition) in &state.transitions {
                writeln!(f, "      -[{}]-> {}", event, transition.to_state)?;
            }
        }

        writeln!(f, "===========================")
    }
}
