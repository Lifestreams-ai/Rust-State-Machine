use serde_json::{Map, Value};
use stateflow::{Action, StateMachine};

/// A test action handler that prints action details for verification.
fn test_action_handler_for_complex(
    action: &Action,
    _memory: &mut Map<String, Value>,
    _context: &mut Context,
) {
    println!(
        "Test executing action: Type: {}, Command: {}",
        action.action_type, action.command
    );
    // Optionally modify the memory if needed
    // For example:
    // memory.insert("last_action".to_string(), Value::String(action.command.clone()));
}

struct Context {}

#[test]
fn test_complex_state_machine() {
    // JSON string literal representing the complex state machine configuration
    let json_config = r#"
    {
        "states": [
            {
                "name": "Idle",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Entering Idle state"
                    }
                ],
                "on_exit_actions": [
                    {
                        "action_type": "log",
                        "command": "Exiting Idle state"
                    }
                ],
                "validations": []
            },
            {
                "name": "Running",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Starting Running state"
                    },
                    {
                        "action_type": "execute",
                        "command": "StartProcess"
                    }
                ],
                "on_exit_actions": [
                    {
                        "action_type": "log",
                        "command": "Stopping Running state"
                    }
                ],
                "validations": []
            },
            {
                "name": "Paused",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Entering Paused state"
                    }
                ],
                "on_exit_actions": [
                    {
                        "action_type": "log",
                        "command": "Exiting Paused state"
                    }
                ],
                "validations": []
            },
            {
                "name": "Completed",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Task completed"
                    }
                ],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "Failed",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Entering Failed state"
                    },
                    {
                        "action_type": "alert",
                        "command": "NotifyAdmin"
                    }
                ],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Idle",
                "event": "Start",
                "to": "Running",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Transitioning from Idle to Running"
                    }
                ],
                "validations": []
            },
            {
                "from": "Running",
                "event": "Pause",
                "to": "Paused",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Pausing the process"
                    }
                ],
                "validations": []
            },
            {
                "from": "Paused",
                "event": "Resume",
                "to": "Running",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Resuming the process"
                    }
                ],
                "validations": []
            },
            {
                "from": "Running",
                "event": "Complete",
                "to": "Completed",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Completing the task"
                    }
                ],
                "validations": []
            },
            {
                "from": "Running",
                "event": "Fail",
                "to": "Failed",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Process failed"
                    }
                ],
                "validations": []
            },
            {
                "from": "Paused",
                "event": "Fail",
                "to": "Failed",
                "actions": [
                    {
                        "action_type": "log",
                        "command": "Process failed while paused"
                    }
                ],
                "validations": []
            }
        ]
    }
    "#;

    // Initialize the memory (empty in this case)
    let memory = Map::new();

    // Initialize the state machine using the complex configuration and the test action handler
    let state_machine = StateMachine::new(
        json_config,
        Some("Idle".to_string()),
        test_action_handler_for_complex,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // Print the initial state of the state machine
    println!("{}", state_machine);

    // Test transitions
    assert!(
        state_machine.trigger("Start").is_ok(),
        "Failed to start the state machine"
    );
    println!("{}", state_machine); // Print the state machine after the transition

    assert!(
        state_machine.trigger("Pause").is_ok(),
        "Failed to pause the state machine"
    );
    println!("{}", state_machine); // Print the state machine after pausing

    assert!(
        state_machine.trigger("Resume").is_ok(),
        "Failed to resume the state machine"
    );
    println!("{}", state_machine); // Print the state machine after resuming

    assert!(
        state_machine.trigger("Complete").is_ok(),
        "Failed to complete the state machine"
    );
    println!("{}", state_machine); // Print the state machine after completing

    // This transition should fail because "Completed" state does not have a "Fail" transition
    assert!(
        state_machine.trigger("Fail").is_err(),
        "Unexpectedly succeeded in failing from a completed state"
    );
    println!("{}", state_machine); // Print the state machine, expect no change due to failed transition
}

/// A test action handler that prints action details for verification.
fn test_action_handler(action: &Action, memory: &mut Map<String, Value>, _context: &mut Context) {
    println!(
        "Test executing action: Type: {}, Command: {}",
        action.action_type, action.command
    );
    // Optionally modify the memory if needed
    // For example, increment a counter in the memory
    if action.action_type == "increment_counter" {
        let counter = memory
            .entry("counter")
            .or_insert_with(|| Value::Number(0.into()));
        if let Value::Number(num) = counter {
            *num = (num.as_i64().unwrap_or(0) + 1).into();
        }
    }
}

/// Test the basic functionality of the state machine with transitions.
#[test]
fn test_basic_transitions() {
    // JSON string representing the state machine configuration
    let json_config = r#"
    {
        "states": [
            {
                "name": "A",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "B",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "A",
                "event": "go_to_b",
                "to": "B",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    let memory = Map::new();

    let state_machine = StateMachine::new(
        json_config,
        Some("A".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    assert_eq!(state_machine.get_current_state().unwrap(), "A");

    // Trigger the transition
    assert!(
        state_machine.trigger("go_to_b").is_ok(),
        "Failed to transition to state B"
    );
    assert_eq!(state_machine.get_current_state().unwrap(), "B");

    // Attempt an invalid transition
    assert!(
        state_machine.trigger("invalid_event").is_err(),
        "Unexpectedly succeeded with an invalid event"
    );
}

/// Test state validations.
#[test]
fn test_state_validations() {
    // JSON configuration with a state validation
    let json_config = r#"
    {
        "states": [
            {
                "name": "Start",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": [
                    {
                        "field": "age",
                        "rules": [
                            { "type": "type_check", "expected_type": "number" },
                            { "type": "min_value", "value": 18 }
                        ]
                    }
                ]
            },
            {
                "name": "End",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Start",
                "event": "proceed",
                "to": "End",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    // Memory with age less than 18
    let mut memory = Map::new();
    memory.insert("age".to_string(), Value::Number(16.into()));

    let state_machine = StateMachine::new(
        json_config,
        Some("Start".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // The state validation should fail
    assert!(
        state_machine.trigger("proceed").is_err(),
        "Unexpectedly succeeded despite failing state validation"
    );

    // Update memory to pass validation
    let mut memory = state_machine.memory.write().unwrap();
    memory.insert("age".to_string(), Value::Number(20.into()));
    drop(memory); // Release the lock

    // Now the transition should succeed
    assert!(
        state_machine.trigger("proceed").is_ok(),
        "Failed to proceed after passing validation"
    );
    assert_eq!(state_machine.get_current_state().unwrap(), "End");
}

/// Test transition validations.
#[test]
fn test_transition_validations() {
    // JSON configuration with a transition validation
    let json_config = r#"
    {
        "states": [
            {
                "name": "Init",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "Processed",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Init",
                "event": "process",
                "to": "Processed",
                "actions": [],
                "validations": [
                    {
                        "field": "approved",
                        "rules": [
                            { "type": "type_check", "expected_type": "boolean" },
                            { "type": "nullable", "is_nullable": false }
                        ]
                    }
                ]
            }
        ]
    }
    "#;

    // Memory without the 'approved' field
    let memory = Map::new();

    let state_machine = StateMachine::new(
        json_config,
        Some("Init".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // The transition validation should fail
    assert!(
        state_machine.trigger("process").is_err(),
        "Unexpectedly succeeded despite failing transition validation"
    );

    // Update memory to pass validation
    let mut memory = state_machine.memory.write().unwrap();
    memory.insert("approved".to_string(), Value::Bool(true));
    drop(memory); // Release the lock

    // Now the transition should succeed
    assert!(
        state_machine.trigger("process").is_ok(),
        "Failed to process after passing validation"
    );
    assert_eq!(state_machine.get_current_state().unwrap(), "Processed");
}

/// Test conditional validations.
#[test]
fn test_conditional_validations() {
    // JSON configuration with conditional validation
    let json_config = r#"
    {
        "states": [
            {
                "name": "Form",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": [
                    {
                        "field": "email",
                        "rules": [
                            { "type": "type_check", "expected_type": "string" }
                        ],
                        "condition": {
                            "field": "email_required",
                            "operator": "==",
                            "value": true
                        }
                    }
                ]
            },
            {
                "name": "Submitted",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Form",
                "event": "submit",
                "to": "Submitted",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    // Memory where email is required but not provided
    let mut memory = Map::new();
    memory.insert("email_required".to_string(), Value::Bool(true));

    let state_machine = StateMachine::new(
        json_config,
        Some("Form".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // Validation should fail
    assert!(
        state_machine.trigger("submit").is_err(),
        "Unexpectedly succeeded despite failing conditional validation"
    );

    // Provide the email
    let mut memory = state_machine.memory.write().unwrap();
    memory.insert(
        "email".to_string(),
        Value::String("user@example.com".to_string()),
    );
    drop(memory); // Release the lock

    // Now the transition should succeed
    assert!(
        state_machine.trigger("submit").is_ok(),
        "Failed to submit after passing conditional validation"
    );
    assert_eq!(state_machine.get_current_state().unwrap(), "Submitted");
}

/// Test memory manipulation within actions without `on_enter_actions` on the start state.
#[test]
fn test_context_manipulation() {
    // JSON configuration with actions that modify the memory
    let json_config = r#"
    {
        "states": [
            {
                "name": "Init",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "Counter",
                "on_enter_actions": [
                    {
                        "action_type": "increment_counter",
                        "command": ""
                    }
                ],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "End",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Init",
                "event": "start",
                "to": "Counter",
                "actions": [],
                "validations": []
            },
            {
                "from": "Counter",
                "event": "finish",
                "to": "End",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    // Initialize memory with counter set to 0
    let mut memory = Map::new();
    memory.insert("counter".to_string(), Value::Number(0.into()));

    let state_machine = StateMachine::new(
        json_config,
        Some("Init".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // The initial state is "Init" with no on_enter_actions

    // Trigger the transition to the "Counter" state
    assert!(
        state_machine.trigger("start").is_ok(),
        "Failed to start counter"
    );

    // The on_enter_action in "Counter" should increment the counter
    {
        let memory = state_machine.memory.read().unwrap();
        let counter = memory.get("counter").and_then(|v| v.as_i64()).unwrap_or(0);
        assert_eq!(
            counter, 1,
            "Counter was not incremented on entering Counter state"
        );
    }

    // Trigger the transition to the "End" state
    assert!(state_machine.trigger("finish").is_ok(), "Failed to finish");

    // Check that the counter remains the same
    {
        let memory = state_machine.memory.read().unwrap();
        let counter = memory.get("counter").and_then(|v| v.as_i64()).unwrap_or(0);
        assert_eq!(
            counter, 1,
            "Counter changed unexpectedly after transitioning to End state"
        );
    }
}

/// Test invalid configuration handling.
#[test]
fn test_invalid_configuration() {
    // JSON configuration missing required fields
    let invalid_json_config = r#"
    {
        "states": [],
        "transitions": []
    }
    "#;

    let memory = Map::new();

    let result = StateMachine::new(
        invalid_json_config,
        None,
        test_action_handler,
        memory,
        Context {},
    );
    assert!(
        result.is_err(),
        "StateMachine initialized with invalid configuration"
    );
}

/// Test saving and restoring state.
#[test]
fn test_state_persistence() {
    // JSON configuration
    let json_config = r#"
    {
        "states": [
            {
                "name": "First",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "Second",
                "on_enter_actions": [],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "First",
                "event": "next",
                "to": "Second",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    let memory = Map::new();

    let state_machine = StateMachine::new(
        json_config,
        Some("First".to_string()),
        test_action_handler,
        memory,
        Context {},
    )
    .expect("Failed to initialize state machine");

    // Transition to the next state
    assert!(
        state_machine.trigger("next").is_ok(),
        "Failed to transition to Second state"
    );

    // Save the current state
    let current_state = state_machine.get_current_state().unwrap();
    assert_eq!(current_state, "Second");

    // Create a new state machine with the saved state
    let new_state_machine = StateMachine::new(
        json_config,
        Some(current_state.clone()),
        test_action_handler,
        Map::new(),
        Context {},
    )
    .expect("Failed to initialize new state machine with saved state");

    // Verify the state
    assert_eq!(new_state_machine.get_current_state().unwrap(), "Second");
}

/// A custom context struct to be used with the state machine.
struct MyContext {
    counter: i32,
}

/// An action handler that uses the context to modify its state.
fn action_handler(action: &Action, _memory: &mut Map<String, Value>, context: &mut MyContext) {
    println!(
        "Executing action: Type: {}, Command: {}",
        action.action_type, action.command
    );
    if action.action_type == "increment_counter" {
        context.counter += 1;
    } else if action.action_type == "reset_counter" {
        context.counter = 0;
    }
}

/// Test case for testing the context usage in the state machine.
#[test]
fn test_context_usage() {
    // JSON configuration with actions that use the context
    let json_config = r#"
    {
        "states": [
            {
                "name": "Init",
                "validations": []
            },
            {
                "name": "Counting",
                "on_enter_actions": [
                    {
                        "action_type": "increment_counter",
                        "command": ""
                    }
                ],
                "on_exit_actions": [],
                "validations": []
            },
            {
                "name": "Reset",
                "on_enter_actions": [
                    {
                        "action_type": "reset_counter",
                        "command": ""
                    }
                ],
                "on_exit_actions": [],
                "validations": []
            }
        ],
        "transitions": [
            {
                "from": "Init",
                "event": "start_counting",
                "to": "Counting",
                "actions": [],
                "validations": []
            },
            {
                "from": "Counting",
                "event": "increment",
                "to": "Counting",
                "actions": [
                    {
                        "action_type": "increment_counter",
                        "command": ""
                    }
                ],
                "validations": []
            },
            {
                "from": "Counting",
                "event": "reset",
                "to": "Reset",
                "actions": [],
                "validations": []
            },
            {
                "from": "Reset",
                "event": "start_counting",
                "to": "Counting",
                "actions": [],
                "validations": []
            }
        ]
    }
    "#;

    // Initialize memory (empty in this case)
    let memory = Map::new();

    // Initialize the context with counter set to 0
    let context = MyContext { counter: 0 };

    // Initialize the state machine using the configuration, memory, and context
    let state_machine = StateMachine::new(
        json_config,
        Some("Init".to_string()),
        action_handler,
        memory,
        context,
    )
    .expect("Failed to initialize state machine");

    // Transition to "Counting" state, which should increment the counter to 1
    assert!(
        state_machine.trigger("start_counting").is_ok(),
        "Failed to start counting"
    );

    // Verify that the context counter is 1
    {
        let context = state_machine.context.read().unwrap();
        assert_eq!(
            context.counter, 1,
            "Counter should be 1 after first increment"
        );
    }

    // Trigger the "increment" event to increment the counter
    assert!(
        state_machine.trigger("increment").is_ok(),
        "Failed to increment counter"
    );

    // Verify that the context counter is 3
    {
        let context = state_machine.context.read().unwrap();
        assert_eq!(
            context.counter, 3,
            "Counter should be 3 after second increment"
        );
    }

    // Reset the counter by transitioning to the "Reset" state
    assert!(
        state_machine.trigger("reset").is_ok(),
        "Failed to reset counter"
    );

    // Verify that the context counter is reset to 0
    {
        let context = state_machine.context.read().unwrap();
        assert_eq!(context.counter, 0, "Counter should be reset to 0");
    }

    // Start counting again
    assert!(
        state_machine.trigger("start_counting").is_ok(),
        "Failed to start counting again"
    );

    // Verify that the context counter is incremented to 1
    {
        let context = state_machine.context.read().unwrap();
        assert_eq!(
            context.counter, 1,
            "Counter should be 1 after restarting counting"
        );
    }
}
