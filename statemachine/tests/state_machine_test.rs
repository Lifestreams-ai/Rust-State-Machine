use statemachine::{Action, StateMachine};

/// A test action handler that prints action details for verification.
fn test_action_handler(action: &Action) {
    println!(
        "Test executing action: Type: {}, Command: {}",
        action.action_type, action.command
    );
}

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
                ]
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
                ]
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
                ]
            },
            {
                "name": "Completed",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Task completed"
                    }
                ],
                "on_exit_actions": []
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
                "on_exit_actions": []
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
                ]
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
                ]
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
                ]
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
                ]
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
                ]
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
                ]
            }
        ]
    }
    "#;

    // Initialize the state machine using the complex configuration and the test action handler
    let state_machine =
        StateMachine::new_with_config(json_config, Some("Idle".to_string()), test_action_handler)
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
