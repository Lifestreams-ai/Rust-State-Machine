# State Machine in Rust

*A simple and extensible state machine implementation in Rust.*

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Lifestreams-ai/statemachine/build.yml?branch=main)](https://github.com/yourusername/Lifestreams-ai/statemachine/actions)
[![Crates.io](https://img.shields.io/crates/v/statemachine-rust.svg)](https://crates.io/crates/statemachine-rust)
[![Rust version](https://img.shields.io/badge/Rust-1.65.0-blue)](https://releases.rs/docs/released/1.65.0)
[![CI](https://github.com/Lifestreams-ai/statemachine/actions/workflows/ci.yml/badge.svg)](https://github.com/Lifestreams-ai/statemachine/actions/workflows/ci.yml)

## Table of Contents

- [State Machine in Rust](#state-machine-in-rust)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Features](#features)
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

This project provides a simple and extensible state machine implementation in Rust. It allows for defining states, transitions, and actions triggered during state changes. The state machine configuration can be loaded from a JSON file, and custom actions can be executed on state transitions using a user-defined action handler.

- **Why this project?** State machines are essential for managing complex state-dependent logic. This library simplifies the creation and management of state machines in Rust applications.

## Features

- **JSON Configuration**: Define states, events, transitions, and actions via a JSON file.
- **On-Enter and On-Exit Actions**: Execute specific actions when entering or exiting a state.
- **Transition Actions**: Perform actions during state transitions.
- **Custom Action Handler**: Implement your own logic for handling actions.
- **Thread-Safe**: Designed with `Arc` and `RwLock` for safe concurrent use.
- **State Persistence**: Save and restore the current state for persistent workflows.
- **Display Trait Implementation**: Visualize the state machine's structure via the `Display` trait.

## Installation

### Prerequisites

- **Rust**: Make sure you have Rust installed. You can install it from [rustup.rs](https://rustup.rs/).

### Steps

1. **Add Dependency**

   Add the following to your `Cargo.toml`:

   ```toml
   [dependencies]
   statemachine-rust = "0.1.0"
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
      "on_exit_actions": []
    },
    {
      "name": "Processing",
      "on_enter_actions": [],
      "on_exit_actions": []
    },
    {
      "name": "Finished",
      "on_enter_actions": [],
      "on_exit_actions": []
    }
  ],
  "transitions": [
    {
      "from": "Idle",
      "event": "start",
      "to": "Processing",
      "actions": []
    },
    {
      "from": "Processing",
      "event": "finish",
      "to": "Finished",
      "actions": []
    }
  ]
}
```

### 2. Implement the Action Handler

Create a function to handle actions:

```rust
use statemachine_rust::{Action, StateMachine};

fn action_handler(action: &Action) {
    match action.action_type.as_str() {
        "log" => println!("Logging: {}", action.command),
        _ => eprintln!("Unknown action: {}", action.command),
    }
}
```

### 3. Initialize the State Machine

```rust
use statemachine_rust::StateMachine;
use std::fs;

fn main() -> Result<(), String> {
    let config_content = fs::read_to_string("config.json")
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let state_machine = StateMachine::new_with_config(
        &config_content,
        None,
        action_handler,
    )?;

    // Trigger events
    state_machine.trigger("start")?;
    state_machine.trigger("finish")?;

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

- **States**: Define each state's `name`, `on_enter_actions`, and `on_exit_actions`.
- **Transitions**: Specify `from` state, `event` triggering the transition, `to` state, and any `actions`.
- **Actions**: Each action includes an `action_type` and a `command`, which the action handler interprets.

Example of an action:

```json
{
  "action_type": "log",
  "command": "Transitioning to Processing state"
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
- **Rust Community**: For the rich ecosystem and support.

## Support

If you have any questions or issues, please open an issue on [GitHub](https://github.com/yourusername/statemachine-rust/issues).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## Roadmap

- **Async Actions**: Support for asynchronous action handling.
- **Visualization Tools**: Generate visual diagrams of the state machine.
- **Enhanced Error Handling**: More descriptive errors and debugging tools.

## FAQ

**Q**: Can I use this library in a multi-threaded environment?

**A**: Yes, the state machine is thread-safe using `Arc` and `RwLock`.

**Q**: How do I handle custom action types?

**A**: Implement your logic within the `action_handler` function based on the `action_type`.

## Code of Conduct

We expect all contributors to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

*This README was generated to provide a comprehensive overview of the State Machine in Rust project. We hope it helps you get started quickly!*
