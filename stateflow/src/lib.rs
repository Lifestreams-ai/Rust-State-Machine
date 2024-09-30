//! A simple state machine implementation in Rust.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, RwLock};

/// Represents an action with a type and command.
#[derive(Debug, Clone)]
pub struct Action {
    /// The type of the action.
    pub action_type: String,
    /// The command to execute.
    pub command: String,
}

/// A struct representing a state and its transitions, including actions on enter and exit.
#[derive(Debug)]
struct State {
    #[allow(dead_code)]
    name: String,
    on_enter_actions: Vec<Action>,
    on_exit_actions: Vec<Action>,
    transitions: HashMap<String, (String, Vec<Action>)>, // Key: event name, Value: (next state name, transition actions)
}

/// Represents the state machine configuration loaded from JSON.
#[derive(Debug, Serialize, Deserialize)]
struct Transition {
    from: String,
    event: String,
    to: String,
    actions: Vec<ActionConfig>, // Actions triggered during the transition
}

#[derive(Debug, Serialize, Deserialize)]
struct StateConfig {
    name: String,
    on_enter_actions: Vec<ActionConfig>,
    on_exit_actions: Vec<ActionConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActionConfig {
    action_type: String,
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StateMachineConfig {
    states: Vec<StateConfig>,
    transitions: Vec<Transition>,
}

/// The state machine containing all states and the current state.
pub struct StateMachine {
    states: Arc<RwLock<HashMap<String, State>>>, // Key: state name, Value: state instance
    current_state: Arc<RwLock<String>>,
    action_handler: Arc<dyn Fn(&Action) + Send + Sync>, // Callback function for handling actions
}

impl StateMachine {
    /// Creates a new state machine from a configuration string, optionally restoring to a specific state.
    /// `initial_state` can be used to restore the machine to a saved state.
    pub fn new_with_config<F>(
        config_content: &str,
        initial_state: Option<String>,
        action_handler: F,
    ) -> Result<Self, String>
    where
        F: Fn(&Action) + Send + Sync + 'static,
    {
        // Parse the configuration from the provided string
        let config: StateMachineConfig = serde_json::from_str(config_content)
            .map_err(|err| format!("Invalid JSON format in configuration: {}", err))?;

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
            };
            states.insert(state_config.name.clone(), state);
        }

        // Populate transitions for each state
        for transition in &config.transitions {
            if let Some(state) = states.get_mut(&transition.from) {
                state.transitions.insert(
                    transition.event.clone(),
                    (
                        transition.to.clone(),
                        Self::create_actions(&transition.actions),
                    ),
                );
            }
        }

        // Determine the starting state: use provided initial state or default to the first state
        let current_state = initial_state.unwrap_or_else(|| config.states[0].name.clone());

        Ok(StateMachine {
            states: Arc::new(RwLock::new(states)),
            current_state: Arc::new(RwLock::new(current_state)),
            action_handler: Arc::new(action_handler),
        })
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
            // Execute on-exit actions
            self.execute_actions(&current_state.on_exit_actions);

            if let Some((next_state_name, transition_actions)) =
                current_state.transitions.get(event)
            {
                // Execute transition actions
                self.execute_actions(transition_actions);

                // Set the new current state
                *self.current_state.write().unwrap() = next_state_name.clone();

                if let Some(next_state) = states.get(next_state_name) {
                    // Execute on-enter actions of the next state
                    self.execute_actions(&next_state.on_enter_actions);
                }

                log::trace!(
                    "Transitioning from {} to {} on event '{}'",
                    current_state_name,
                    next_state_name,
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
    fn execute_actions(&self, actions: &[Action]) {
        for action in actions {
            (self.action_handler)(action); // Call the user-provided callback with the action
        }
    }

    /// Saves the current state of the state machine and returns it as a string.
    pub fn save_state(&self) -> Result<String, String> {
        let current_state = self.current_state.read().unwrap();
        serde_json::to_string(&*current_state)
            .map_err(|err| format!("Failed to serialize current state: {}", err))
    }
}

/// Implementing the Display trait to render the state machine as a string.
impl Display for StateMachine {
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
            writeln!(f, "{} State: {}", marker, state_name)?;

            for (event, (to_state, _)) in &state.transitions {
                writeln!(f, "      -[{}]-> {}", event, to_state)?;
            }
        }

        writeln!(f, "======================")
    }
}
