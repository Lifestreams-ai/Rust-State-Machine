use serde_json::{json, Map, Value};
use stateflow::{Action, AsyncStateMachine};
use std::time::Duration;
use tokio::time::sleep;

struct Context {}
#[tokio::main]
async fn main() -> Result<(), String> {
    // Define your state machine configuration
    let config = r#"{
        "states": [
            {
                "name": "start",
                "on_enter_actions": [
                    {
                        "action_type": "log",
                        "command": "Entering start state"
                    }
                ]
            },
            {
                "name": "processing",
                "on_enter_actions": [
                    {
                        "action_type": "process",
                        "command": "Processing data"
                    }
                ]
            }
        ],
        "transitions": [
            {
                "from": "start",
                "event": "begin_process",
                "to": "processing"
            }
        ]
    }"#;

    // Create an async action handler that takes owned values
    let action_handler =
        |action: Action, memory: &mut Map<String, Value>, _context: &mut Context| {
            Box::pin(async move {
                match action.action_type.as_str() {
                    "log" => println!("Log: {}", action.command),
                    "process" => {
                        println!("Starting: {}", action.command);
                        sleep(Duration::from_secs(1)).await;
                        println!("Completed: {}", action.command);
                        memory.insert("processed".to_string(), json!(true));
                    }
                    _ => println!("Unknown action type: {}", action.action_type),
                }
            })
        };

    // Initialize the state machine
    let machine = AsyncStateMachine::new(config, None, action_handler, Map::new(), Context {})?;

    // Trigger an event
    machine.trigger("begin_process").await?;

    Ok(())
}
