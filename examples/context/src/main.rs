//! This example demonstrates how to use a custom context struct with the state machine.
//!
use serde_json::{Map, Value};
use stateflow::{Action, StateMachine};

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

fn main() {
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
