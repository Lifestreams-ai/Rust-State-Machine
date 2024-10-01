//! This example demonstrates a complex state machine configuration with multiple states, transitions, and actions.

use serde_json::{Map, Value};
use stateflow::{Action, StateMachine};

/// An action handler that prints action details.
fn action_handler(action: &Action, _context: &mut Map<String, Value>) {
    println!(
        "Executing action: Type: {}, Command: {}",
        action.action_type, action.command
    );
    // Optionally modify the context if needed
    // For example:
    // context.insert("last_action".to_string(), Value::String(action.command.clone()));
}

fn main() -> Result<(), String> {
    // JSON string representing the complex state machine configuration
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

    // Initialize the context (empty in this case)
    let context = Map::new();

    // Initialize the state machine using the configuration, context, and the action handler
    let state_machine = StateMachine::new(
        json_config,
        Some("Idle".to_string()),
        action_handler,
        context,
    )
    .expect("Failed to initialize state machine");

    // Print the initial state of the state machine
    println!("{}", state_machine);

    // Trigger events and handle results
    state_machine.trigger("Start")?;
    println!("{}", state_machine);

    state_machine.trigger("Pause")?;
    println!("{}", state_machine);

    state_machine.trigger("Resume")?;
    println!("{}", state_machine);

    state_machine.trigger("Complete")?;
    println!("{}", state_machine);

    // This transition should fail because "Completed" state does not have a "Fail" transition
    if let Err(e) = state_machine.trigger("Fail") {
        println!("Expected failure: {}", e);
    } else {
        println!("Unexpectedly succeeded in failing from a completed state");
    }
    println!("{}", state_machine);

    Ok(())
}
