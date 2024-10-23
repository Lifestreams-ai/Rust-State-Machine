<!-- omit in toc -->
# Contributing to Stateflow

First off, thank you for considering contributing to **Stateflow**! Your contributions are highly appreciated and will help make this project even better.

This guide will help you understand how to contribute to the project, from reporting issues to submitting code changes.

---

<!-- omit in toc -->
## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Improving Documentation](#improving-documentation)
  - [Submitting Pull Requests](#submitting-pull-requests)
- [Development Guidelines](#development-guidelines)
  - [Setting Up the Development Environment](#setting-up-the-development-environment)
  - [Coding Standards](#coding-standards)
  - [Commit Messages](#commit-messages)
  - [Testing](#testing)
- [License](#license)
- [Contact](#contact)

---

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](stateflow/CODE_OF_CONDUCT.md). By participating in this project, you agree to abide by its terms.

## How Can I Contribute?

### Reporting Bugs

If you find a bug in the project, please open an issue on GitHub. Before reporting, please check the existing issues to see if it has already been reported.

**To report a bug:**

1. **Search Existing Issues**: Ensure the issue hasn't already been reported.
2. **Create a New Issue**: Use the [bug report template](https://github.com/Lifestreams-ai/statemachine/issues/new?template=bug_report.md).
3. **Provide Detailed Information**:
   - **Title**: A clear and descriptive title.
   - **Description**: A detailed description of the issue.
   - **Steps to Reproduce**: Include the steps to reproduce the problem.
   - **Expected Behavior**: What you expected to happen.
   - **Actual Behavior**: What actually happened.
   - **Environment**: Include details about your environment (OS, Rust version, etc.).
4. **Screenshots/Logs**: Attach any relevant screenshots or logs.

### Suggesting Features

We welcome new ideas to improve the project.

**To suggest a feature:**

1. **Check for Existing Issues**: See if someone else has already suggested it.
2. **Open a New Issue**: Use the [feature request template](https://github.com/Lifestreams-ai/statemachine/issues/new?template=feature_request.md).
3. **Provide a Detailed Description**:
   - **Title**: A concise title for the feature.
   - **Description**: A detailed explanation of the feature.
   - **Use Cases**: Explain how this feature would be useful.
   - **Possible Implementation**: If you have ideas on how to implement it, share them.

### Improving Documentation

Good documentation helps others understand and use the project.

**Ways to contribute:**

- Fix typos or grammatical errors.
- Clarify existing documentation.
- Add examples or tutorials.
- Translate documentation to other languages.

**To contribute to documentation:**

1. **Fork the Repository**.
2. **Make Your Changes** in the `docs/` directory or relevant Markdown files.
3. **Submit a Pull Request** following the guidelines below.

### Submitting Pull Requests

We appreciate code contributions that fix bugs or add new features.

**To submit a pull request:**

1. **Fork the Repository**.
2. **Clone Your Fork**:

   ```bash
   git clone https://github.com/yourusername/stateflow.git
   ```

3. **Create a Feature Branch**:

   ```bash
   git checkout -b feature/YourFeature
   ```

4. **Make Your Changes**.
5. **Ensure All Tests Pass**.
6. **Commit Your Changes**:

   ```bash
   git commit -m "Description of your changes"
   ```

7. **Push to Your Fork**:

   ```bash
   git push origin feature/YourFeature
   ```

8. **Open a Pull Request**:

   - Go to the original repository.
   - Click on "New Pull Request".
   - Select your branch and provide a clear description.

9. **Address Review Comments**: Be prepared to make changes based on feedback.

---

## Development Guidelines

### Setting Up the Development Environment

**Prerequisites:**

- **Rust**: Install via [rustup.rs](https://rustup.rs/).
- **Cargo**: Comes with Rust installation.
- **Clippy**: For linting. Install with `rustup component add clippy`.

**Steps:**

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/Lifestreams-ai/statemachine.git
   ```

2. **Navigate to the Project Directory**:

   ```bash
   cd stateflow
   ```

3. **Build the Project**:

   ```bash
   cargo build
   ```

4. **Run Tests**:

   ```bash
   cargo test
   ```

5. **Run Lints**:

   ```bash
   cargo clippy
   ```

### Coding Standards

- **Follow Rust Style Guidelines**: Use `rustfmt` to format your code.

  ```bash
  rustup component add rustfmt
  cargo fmt
  ```

- **Write Clear and Concise Code**: Prioritize readability.
- **Document Your Code**: Use `///` for public items and `//` for internal comments.
- **Error Handling**: Use `Result` and `Option` types appropriately.

### Commit Messages

- **Use Present Tense**: "Add feature" not "Added feature".
- **Be Descriptive**: Explain what and why, not how.
- **Reference Issues**: If applicable, reference related issues or pull requests.

**Example Commit Message:**

```
Add asynchronous action handling

Implemented support for asynchronous action execution in the state machine.
This allows actions to perform non-blocking operations.

Closes #45
```

### Testing

- **Write Unit Tests**: For any new features or bug fixes.
- **Run All Tests Before Submitting**:

  ```bash
  cargo test
  ```

- **Test Async Code**: Use `#[tokio::test]` for asynchronous tests.

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).

## Contact

If you have any questions, feel free to reach out:

- **Email**: [contact@stateflow.com](mailto:contact@stateflow.com)
- **GitHub Issues**: Open an issue for any general questions.

---

*Thank you for your interest in contributing to Stateflow! Together, we can make it better.*