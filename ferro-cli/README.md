# Ferro CLI

A CLI tool for scaffolding Ferro web applications.

## Installation

```bash
cargo install ferro-cli
```

## Usage

### Create a new project

```bash
ferro new myapp
```

This will interactively prompt you for:
- Project name
- Description
- Author

### Non-interactive mode

```bash
ferro new myapp --no-interaction
```

### Skip git initialization

```bash
ferro new myapp --no-git
```

## Generated Project Structure

```
myapp/
├── Cargo.toml
├── .gitignore
└── src/
    ├── main.rs
    └── controllers/
        ├── mod.rs
        └── home.rs
```

## License

MIT
