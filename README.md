<!-- omit in toc -->
# State Machine in Rust

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/stateflow.svg)](https://crates.io/crates/stateflow)
[![Rust version](https://img.shields.io/badge/Rust-1.81.0-blue)](https://releases.rs/docs/1.81.0/)
[![CI](https://github.com/Lifestreams-ai/statemachine/actions/workflows/ci.yml/badge.svg)](https://github.com/Lifestreams-ai/statemachine/actions/workflows/ci.yml)
<!-- [![Build Status](https://img.shields.io/github/actions/workflow/status/Lifestreams-ai/statemachine/build.yml?branch=main)](https://github.com/yourusername/Lifestreams-ai/statemachine/actions) -->

A simple and extensible state machine implementation in Rust.

<!-- omit in toc -->
## Features

- **JSON Configuration**: Define states, events, transitions, actions, and validations via a JSON file.
- **On-Enter and On-Exit Actions**: Execute specific actions when entering or exiting a state.
- **Transition Actions**: Perform actions during state transitions.
- **Custom Action Handler**: Implement your own logic for handling actions, with access to both memory and custom context.
- **Thread-Safe**: Designed with `Arc` and `RwLock` for safe concurrent use.
- **State Persistence**: Save and restore the current state for persistent workflows.
- **Display Trait Implementation**: Visualize the state machine's structure via the `Display` trait.
- **Data-Driven Validations**: Define validation rules in the configuration to enforce constraints on memory.
- **Conditional Validations**: Apply validations conditionally based on memory values.
- **Custom Context Support**: Pass a custom context object to the state machine, accessible in the action handler.

<!-- omit in toc -->
## Overview

- [Introduction](#introduction)
- [Installation](#installation)
  - [Prerequisites](#prerequisites)
  - [Steps](#steps)
- [Usage](#usage)
  - [1. Define Your Configuration](#1-define-your-configuration)
  - [2. Implement the Action Handler](#2-implement-the-action-handler)
  - [3. Initialize the State Machine](#3-initialize-the-state-machine)
  - [4. Run Your Application](#4-run-your-application)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)
- [Credits](#credits)
- [Support](#support)
- [Changelog](#changelog)
- [Roadmap](#roadmap)
- [FAQ](#faq)
- [Code of Conduct](#code-of-conduct)

## Introduction

This project provides a simple and extensible state machine implementation in Rust. It allows for defining states, transitions, actions, and validations triggered during state changes. The state machine configuration can be loaded from a JSON file, and custom actions can be executed on state transitions using a user-defined action handler. The state machine also supports custom context objects, allowing you to maintain additional state or data throughout the state machine's lifecycle.

- **Why this project?** State machines are essential for managing complex state-dependent logic. This library simplifies the creation and management of state machines in Rust applications, providing flexibility and extensibility.

## Installation

### Prerequisites

- **Rust**: Make sure you have Rust installed. You can install it from [rustup.rs](https://rustup.rs/).

### Steps

1. **Add Dependency**

   Add the following to your `Cargo.toml`:

   ```toml
   [dependencies]
   statemachine-rust = "0.2.0"
   ```

2. **Update Crates**

   Run:

   ```bash
   cargo update
   ```

## Usage

Here's how to integrate the state machine into your project.

### 1. Define Your Configuration

Create a `config.json` file:

```json
{
  "states": [
    {
      "name": "Idle",
      "on_enter_actions": [],
      "on_exit_actions": [],
      "validations": []
    },
    {
      "name": "Processing",
      "on_enter_actions": [],
      "on_exit_actions": [],
      "validations": []
    },
    {
      "name": "Finished",
      "on_enter_actions": [],
      "on_exit_actions": [],
      "validations": []
    }
  ],
  "transitions": [
    {
      "from": "Idle",
      "event": "start",
      "to": "Processing",
      "actions": [],
      "validations": []
    },
    {
      "from": "Processing",
      "event": "finish",
      "to": "Finished",
      "actions": [],
      "validations": []
    }
  ]
}
```

### 2. Implement the Action Handler

Create a function to handle actions, with access to both the memory and your custom context:

```rust
use stateflow::{Action, StateMachine};
use serde_json::{Map, Value};

struct MyContext {
    // Your custom context fields
    counter: i32,
}

fn action_handler(action: &Action, memory: &mut Map<String, Value>, context: &mut MyContext) {
    match action.action_type.as_str() {
        "log" => println!("Logging: {}", action.command),
        "increment_counter" => {
            context.counter += 1;
            println!("Counter incremented to {}", context.counter);
        },
        _ => eprintln!("Unknown action: {}", action.command),
    }
}
```

### 3. Initialize the State Machine

```rust
use stateflow::StateMachine;
use serde_json::{Map, Value};
use std::fs;

fn main() -> Result<(), String> {
    let config_content = fs::read_to_string("config.json")
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    // Initialize memory (can be empty or pre-populated)
    let memory = Map::new();

    // Initialize your custom context
    let context = MyContext { counter: 0 };

    let state_machine = StateMachine::new(
        &config_content,
        Some("Idle".to_string()),
        action_handler,
        memory,
        context,
    )?;

    // Trigger events
    state_machine.trigger("start")?;
    state_machine.trigger("finish")?;

    // Access the context after transitions
    {
        let context = state_machine.context.read().unwrap();
        println!("Final counter value: {}", context.counter);
    }

    Ok(())
}
```

### 4. Run Your Application

Compile and run your application:

```bash
cargo run
```

## Configuration

The state machine is highly configurable via a JSON file.

- **States**: Define each state's `name`, `on_enter_actions`, `on_exit_actions`, and `validations`.
- **Transitions**: Specify `from` state, `event` triggering the transition, `to` state, any `actions`, and `validations`.
- **Actions**: Each action includes an `action_type` and a `command`, which the action handler interprets.
- **Validations**: Define validation rules to enforce constraints on memory fields, with optional conditions.

Example of a state with validations:

```json
{
  "name": "Processing",
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
}
```

Example of a transition with an action:

```json
{
  "from": "Idle",
  "event": "start",
  "to": "Processing",
  "actions": [
    {
      "action_type": "log",
      "command": "Transitioning to Processing state"
    }
  ],
  "validations": []
}
```

## Contributing

We welcome contributions!

1. **Fork the Repository**
2. **Create a Feature Branch**

   ```bash
   git checkout -b feature/YourFeature
   ```

3. **Commit Your Changes**
4. **Push to Your Branch**

   ```bash
   git push origin feature/YourFeature
   ```

5. **Open a Pull Request**

For detailed guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Credits

- **[Serde](https://serde.rs/)**: For serialization and deserialization.
- **[JSON Schema](https://json-schema.org/)**: For configuration validation.
- **Rust Community**: For the rich ecosystem and support.

## Support

If you have any questions or issues, please open an issue on [GitHub](https://github.com/Lifestreams-ai/stateflow/issues).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Roadmap

- **Async Actions**: Support for asynchronous action handling.
- **Visualization Tools**: Generate visual diagrams of the state machine.
- **Enhanced Error Handling**: More descriptive errors and debugging tools.
- **Extended Validations**: Support for more complex validation rules.

## FAQ

**Q**: Can I use this library in a multi-threaded environment?

**A**: Yes, the state machine is thread-safe using `Arc` and `RwLock`.

**Q**: How do I handle custom action types?

**A**: Implement your logic within the `action_handler` function based on the `action_type`.

**Q**: Can I pass my own context to the state machine?

**A**: Yes, you can pass a custom context of any type to the state machine, which is accessible in the action handler.

## Code of Conduct

We expect all contributors to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

*This README was updated to provide a comprehensive overview of the State Machine in Rust project. We hope it helps you get started quickly!*
