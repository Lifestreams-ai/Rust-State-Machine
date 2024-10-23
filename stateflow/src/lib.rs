//! A simple state machine library for Rust.

use lru::LruCache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::num::NonZero;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock; // Alias to differentiate

/// Represents an action with a type and command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The type of the action.
    pub action_type: String,
    /// The command to execute.
    pub command: String,
}

/// A struct representing a state and its transitions, including actions on enter and exit.
#[derive(Debug, Clone)]
struct State {
    name: String,
    on_enter_actions: Vec<Action>,
    on_exit_actions: Vec<Action>,
    transitions: HashMap<String, Transition>, // Key: event name, Value: Transition instance
    validations: Vec<ValidationRule>,         // State validation rules
}

/// Represents a transition between states, including actions and validations.
#[derive(Debug, Clone)]
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

type ActionHandler<C> = dyn for<'a> Fn(
        &'a Action,
        &'a mut Map<String, Value>,
        &'a mut C,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    + Send
    + Sync;

/// Define environment variable name and default cache size
const LRU_CACHE_SIZE_ENV_KEY: &str = "STATEFLOW_LRU_CACHE_SIZE";
const DEFAULT_CACHE_SIZE: usize = 100;

/// Retrieves the LRU cache size from the environment variable.
/// Defaults to `DEFAULT_CACHE_SIZE` if not set or invalid.
fn get_cache_size() -> usize {
    let lru_cache_size_env: usize = env::var(LRU_CACHE_SIZE_ENV_KEY)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_CACHE_SIZE);
    if lru_cache_size_env == 0 {
        DEFAULT_CACHE_SIZE
    } else {
        lru_cache_size_env
    }
}

/// Static cache for storing parsed configurations
static CONFIG_CACHE: Lazy<RwLock<LruCache<u64, Arc<StateMachineConfig>>>> = Lazy::new(|| {
    let cache_size = get_cache_size();
    RwLock::new(LruCache::new(NonZero::new(cache_size).unwrap()))
});

/// The state machine containing all states, the current state, memory, context, and handlers.
pub struct StateMachine<'a, C> {
    states: Arc<RwLock<HashMap<String, State>>>,
    current_state: Arc<RwLock<String>>,
    action_handler: Arc<ActionHandler<C>>,
    /// The memory used by the state machine to store data.
    pub memory: Arc<AsyncRwLock<Map<String, Value>>>,
    /// The context used by the state machine to store state.
    pub context: Arc<AsyncRwLock<C>>,
    _marker: std::marker::PhantomData<&'a ()>, // To tie the lifetime to the struct
}

impl<'a, C> StateMachine<'a, C> {
    /// Creates a new state machine from a JSON configuration string.
    pub fn new<F>(
        config_content: &str,
        initial_state: Option<String>,
        action_handler: F,
        memory: Map<String, Value>,
        context: C,
    ) -> Result<Self, String>
    where
        F: for<'b> Fn(
                &'b Action,
                &'b mut Map<String, Value>,
                &'b mut C,
            ) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'b>>
            + Send
            + Sync
            + 'static,
    {
        // Compute the hash of the config_content
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        config_content.hash(&mut hasher);
        let config_hash = hasher.finish();

        // Try to get the cached config
        let config: Arc<StateMachineConfig> = {
            let mut cache = CONFIG_CACHE.write().unwrap();
            if let Some(cached_config) = cache.get(&config_hash) {
                cached_config.clone()
            } else {
                // Parse and validate the config
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
                let config_deserialized: StateMachineConfig = serde_json::from_value(config_value)
                    .map_err(|err| format!("Failed to deserialize configuration: {}", err))?;

                // Validate the config
                Self::validate_config(&config_deserialized)?;

                // Cache the config
                let config_arc = Arc::new(config_deserialized);
                cache.put(config_hash, config_arc.clone());
                config_arc
            }
        };

        // Now proceed to create the StateMachine using `config`
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
            memory: Arc::new(AsyncRwLock::new(memory)),
            context: Arc::new(AsyncRwLock::new(context)),
            _marker: std::marker::PhantomData,
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
    pub async fn trigger(&self, event: &str) -> Result<(), String> {
        // Acquire a read lock on the current state and clone its value
        let current_state_name = {
            let current_state_guard = self.current_state.read().unwrap();
            current_state_guard.clone()
        }; // Lock is released here

        // Acquire a read lock on the states and get the current state and transition
        let (current_state, transition) = {
            let states_guard = self.states.read().unwrap();
            // Clone the current state to own its data
            let current_state = states_guard.get(&current_state_name).cloned();
            if let Some(current_state) = current_state {
                // Clone the transition to own its data
                if let Some(transition) = current_state.transitions.get(event).cloned() {
                    (current_state, transition)
                } else {
                    return Err(format!(
                        "No transition found for event '{}' from state '{}'.",
                        event, current_state_name
                    ));
                }
            } else {
                return Err(format!(
                    "Current state '{}' not found in state machine.",
                    current_state_name
                ));
            }
        }; // Lock is released here

        // Now `current_state` and `transition` own their data and do not borrow from `states_guard`

        // Acquire write locks on memory and context
        let mut memory = self.memory.write().await;
        let mut context = self.context.write().await;

        // Execute state validations
        Self::evaluate_validations(&current_state.validations, &memory)?;

        // Execute transition validations
        Self::evaluate_validations(&transition.validations, &memory)?;

        // Execute on-exit actions
        self.execute_actions(&current_state.on_exit_actions, &mut memory, &mut context)
            .await;

        // Execute transition actions
        self.execute_actions(&transition.actions, &mut memory, &mut context)
            .await;

        // Update the current state
        {
            let mut current_state_guard = self.current_state.write().unwrap();
            *current_state_guard = transition.to_state.clone();
        } // Lock is released here

        // Execute on-enter actions of the next state
        let next_state_on_enter_actions = {
            let states_guard = self.states.read().unwrap();
            if let Some(next_state) = states_guard.get(&transition.to_state) {
                next_state.on_enter_actions.clone()
            } else {
                return Err(format!(
                    "Next state '{}' not found in state machine.",
                    transition.to_state
                ));
            }
        }; // Lock is released here

        // Now we can call execute_actions with the cloned actions
        self.execute_actions(&next_state_on_enter_actions, &mut memory, &mut context)
            .await;

        Ok(())
    }

    /// Executes a list of actions using the provided async action handler.
    async fn execute_actions<'b>(
        &self,
        actions: &[Action],
        memory: &'b mut Map<String, Value>,
        context: &'b mut C,
    ) {
        for action in actions {
            (self.action_handler)(action, memory, context).await;
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
    pub async fn get_current_state(&self) -> Result<String, String> {
        let current_state_guard = self.current_state.read().unwrap();
        Ok(current_state_guard.clone())
    }
}

/// Implementing the Display trait to render the state machine as a string.
impl<'a, C> Display for StateMachine<'a, C> {
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
