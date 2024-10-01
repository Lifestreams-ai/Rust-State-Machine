//! This example demonstrates how to create a simple state machine with validations and actions.

use serde_json::{Map, Value};
use stateflow::{Action, StateMachine};

fn main() -> Result<(), String> {
    let config_content = r#"
        {
        "states": [
            {
            "name": "Start",
            "on_enter_actions": [],
            "on_exit_actions": [
                {
                    "action_type": "log",
                    "command": "Exiting Idle state"
                }
            ],
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
            "on_exit_actions": []
            }
        ],
        "transitions": [
            {
            "from": "Start",
            "event": "proceed",
            "to": "End",
            "actions": [],
            "validations": [
                {
                "field": "consent",
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
    let mut context = Map::new();
    context.insert("age".to_string(), Value::from(20));
    context.insert("consent".to_string(), Value::from(true));

    let action_handler = |action: &Action, _context: &mut Map<String, Value>| {
        // Handle actions, possibly modifying context
        println!("Action: {:?}", action);
    };

    let state_machine = StateMachine::new(config_content, None, action_handler, context)?;

    // Now you can trigger events and the validations will be applied
    state_machine.trigger("proceed")?;

    Ok(())
}
