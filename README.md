# Mento AI - Statemachine

This project provides a simple and extensible state machine implementation in Rust. It allows for defining states, transitions, and actions triggered during state changes. The state machine configuration can be loaded from a JSON file, and custom actions can be executed on state transitions, using a user-defined action handler. It supports defining on-enter and on-exit actions for each state, as well as transition actions between states. The state machine is thread-safe, making use of Arc and RwLock for managing state transitions in a concurrent environment. Additionally, the current state can be saved and restored for persistent workflows.

### Test

```
cargo make test
``` 
