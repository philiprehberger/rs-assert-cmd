# rs-assert-cmd

[![Crates.io](https://img.shields.io/crates/v/philiprehberger-assert-cmd)](https://crates.io/crates/philiprehberger-assert-cmd)
[![docs.rs](https://img.shields.io/docsrs/philiprehberger-assert-cmd)](https://docs.rs/philiprehberger-assert-cmd)
[![CI](https://github.com/philiprehberger/rs-assert-cmd/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-assert-cmd/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Ergonomic CLI binary integration testing with fluent assertions on stdout, stderr, and exit code

## Requirements

- Rust 1.70+

## Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
philiprehberger-assert-cmd = "0.1.1"
```

## Usage

```rust
use philiprehberger_assert_cmd::cmd;

// Basic command assertion
cmd("echo")
    .arg("hello world")
    .run()
    .unwrap()
    .assert_success()
    .assert_stdout_contains("hello");

// Chain multiple assertions
cmd("rustc")
    .arg("--version")
    .run()
    .unwrap()
    .assert_success()
    .assert_exit_code(0)
    .assert_stdout_contains("rustc")
    .assert_stderr_is_empty();

// Environment variables and stdin
cmd("cat")
    .stdin("input data\n")
    .run()
    .unwrap()
    .assert_success()
    .assert_stdout_equals("input data\n");

// Glob pattern matching
cmd("echo")
    .arg("version 1.2.3")
    .run()
    .unwrap()
    .assert_stdout_matches("version *");
```

## API

### Builder

| Method | Description |
|---|---|
| `cmd(program)` | Create a new command builder |
| `Cmd::new(program)` | Create a new command builder |
| `.arg(arg)` | Append a single argument |
| `.args(&[args])` | Append multiple arguments |
| `.env(key, value)` | Set an environment variable |
| `.stdin(data)` | Pipe data to stdin |
| `.current_dir(dir)` | Set the working directory |
| `.timeout(duration)` | Set a timeout for execution |
| `.run()` | Execute and return `Result<CmdOutput, CmdError>` |

### Assertions

| Method | Description |
|---|---|
| `.assert_success()` | Exit code is 0 |
| `.assert_failure()` | Exit code is non-zero |
| `.assert_exit_code(code)` | Exit code matches exactly |
| `.assert_stdout_contains(s)` | Stdout contains substring |
| `.assert_stdout_equals(s)` | Stdout equals exactly |
| `.assert_stdout_is_empty()` | Stdout is empty |
| `.assert_stdout_line_count(n)` | Stdout has n lines |
| `.assert_stdout_matches(pat)` | Stdout matches glob pattern (`*`, `?`) |
| `.assert_stderr_contains(s)` | Stderr contains substring |
| `.assert_stderr_equals(s)` | Stderr equals exactly |
| `.assert_stderr_is_empty()` | Stderr is empty |
| `.stdout_lines()` | Split stdout into `Vec<&str>` |

## Development

```bash
cargo build
cargo test
cargo clippy -- -D warnings
```

## License

MIT
