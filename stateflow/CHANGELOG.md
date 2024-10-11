# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

- Add asynchronous action handling support.
- Implement conditional validations based on memory values.
- Enhance the `Display` trait for better state machine visualization.
- Improve error messages for validation failures.
- Add support for custom context objects accessible in action handlers.
- Refactor locking mechanism to use synchronous locks where appropriate.

## [0.2.0] - 2024-04-25

### Added

- **Custom Context Support**: Ability to pass a custom context object to the state machine, accessible in the action handler.
- **Conditional Validations**: Apply validations conditionally based on memory values.
- **Display Trait Implementation**: Implemented the `Display` trait to visualize the state machine's structure.
- **Tests**: Added comprehensive tests covering various aspects of the state machine, including validations and context manipulation.

### Fixed

- **Lifetime Issues**: Resolved lifetime mismatches in the `action_handler` by adjusting the `ActionHandler` type and ensuring `'static` lifetimes where necessary.
- **Concurrency**: Improved thread safety by refining the locking strategy, using synchronous locks for certain fields to prevent deadlocks.

## [0.1.0] - 2024-03-15

### Added

- **JSON Configuration**: Ability to define states, events, transitions, actions, and validations via a JSON file.
- **On-Enter and On-Exit Actions**: Execute specific actions when entering or exiting a state.
- **Transition Actions**: Perform actions during state transitions.
- **Custom Action Handler**: Implement custom logic for handling actions, with access to both memory and custom context.
- **Thread-Safe Design**: Utilized `Arc` and `RwLock` to ensure safe concurrent use.
- **State Persistence**: Feature to save and restore the current state for persistent workflows.
- **Data-Driven Validations**: Define validation rules in the configuration to enforce constraints on memory.
- **Basic Tests**: Initial set of tests to verify state transitions and action handling.

### Fixed

- N/A

## [0.0.1] - 2024-02-20

### Added

- **Initial Release**: Basic state machine implementation with JSON configuration support.
- **State Definitions**: Define states with optional on-enter and on-exit actions.
- **Transition Definitions**: Specify transitions between states based on events.
- **Action Handling**: Execute actions during state transitions.

### Fixed

- N/A