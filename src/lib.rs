//! Ergonomic CLI binary integration testing with fluent assertions on stdout, stderr, and exit code.
//!
//! `philiprehberger-assert-cmd` provides a fluent builder for spawning CLI processes and
//! asserting on their output. Zero dependencies — uses only `std::process::Command`.
//!
//! # Example
//!
//! ```no_run
//! use philiprehberger_assert_cmd::cmd;
//!
//! cmd("echo")
//!     .arg("hello")
//!     .run()
//!     .unwrap()
//!     .assert_success()
//!     .assert_stdout_contains("hello");
//! ```

use std::io::Write;
use std::process::Command;
use std::time::Duration;

/// Errors that can occur when running a command.
#[derive(Debug)]
pub enum CmdError {
    /// The process failed to spawn.
    SpawnFailed(std::io::Error),
    /// The process exceeded the configured timeout.
    Timeout,
    /// An assertion on the command output failed.
    AssertionFailed {
        /// What was being checked.
        context: String,
        /// The expected value.
        expected: String,
        /// The actual value.
        actual: String,
    },
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdError::SpawnFailed(e) => write!(f, "failed to spawn process: {}", e),
            CmdError::Timeout => write!(f, "process timed out"),
            CmdError::AssertionFailed {
                context,
                expected,
                actual,
            } => write!(
                f,
                "assertion failed: {}\n  expected: {}\n  actual:   {}",
                context, expected, actual
            ),
        }
    }
}

impl std::error::Error for CmdError {}

/// A builder for configuring and running a CLI command.
///
/// Use [`cmd`] or [`Cmd::new`] to create a new instance, then chain builder
/// methods before calling [`Cmd::run`].
pub struct Cmd {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    stdin_data: Option<String>,
    current_dir: Option<String>,
    timeout: Option<Duration>,
}

/// Create a new [`Cmd`] builder for the given program.
///
/// This is a convenience function equivalent to [`Cmd::new`].
///
/// # Example
///
/// ```no_run
/// use philiprehberger_assert_cmd::cmd;
///
/// let output = cmd("rustc").arg("--version").run().unwrap();
/// output.assert_success();
/// ```
pub fn cmd(program: &str) -> Cmd {
    Cmd::new(program)
}

impl Cmd {
    /// Create a new `Cmd` builder for the given program.
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
            env: Vec::new(),
            stdin_data: None,
            current_dir: None,
            timeout: None,
        }
    }

    /// Append a single argument to the command.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Append multiple arguments to the command.
    pub fn args(mut self, args: &[&str]) -> Self {
        for a in args {
            self.args.push((*a).to_string());
        }
        self
    }

    /// Set an environment variable for the command.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_string(), value.to_string()));
        self
    }

    /// Provide data to pipe into the command's stdin.
    pub fn stdin(mut self, data: impl Into<String>) -> Self {
        self.stdin_data = Some(data.into());
        self
    }

    /// Set the working directory for the command.
    pub fn current_dir(mut self, dir: impl Into<String>) -> Self {
        self.current_dir = Some(dir.into());
        self
    }

    /// Set a timeout for the command. If the process exceeds this duration,
    /// [`CmdError::Timeout`] is returned.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Execute the command and return a [`CmdOutput`] on success.
    pub fn run(&self) -> Result<CmdOutput, CmdError> {
        let mut command = Command::new(&self.program);
        command.args(&self.args);

        for (key, value) in &self.env {
            command.env(key, value);
        }

        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }

        if self.stdin_data.is_some() {
            command.stdin(std::process::Stdio::piped());
        }

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut child = command.spawn().map_err(CmdError::SpawnFailed)?;

        if let Some(data) = &self.stdin_data {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(data.as_bytes());
            }
            // Drop stdin to signal EOF
            child.stdin.take();
        }

        if let Some(timeout) = self.timeout {
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            return Err(CmdError::Timeout);
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => return Err(CmdError::SpawnFailed(e)),
                }
            }
        }

        let output = child.wait_with_output().map_err(CmdError::SpawnFailed)?;

        let status = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CmdOutput {
            status,
            stdout,
            stderr,
        })
    }
}

/// The result of running a command, with chainable assertion methods.
///
/// All assertion methods return `&Self` so they can be chained. They panic
/// with a descriptive message if the assertion fails.
#[derive(Debug, Clone)]
pub struct CmdOutput {
    /// The exit code of the process.
    pub status: i32,
    /// The captured standard output.
    pub stdout: String,
    /// The captured standard error.
    pub stderr: String,
}

impl CmdOutput {
    /// Assert that the command exited with code 0.
    pub fn assert_success(&self) -> &Self {
        if self.status != 0 {
            panic!(
                "assertion failed: expected success (exit code 0), got exit code {}\nstdout: {}\nstderr: {}",
                self.status, self.stdout, self.stderr
            );
        }
        self
    }

    /// Assert that the command exited with a non-zero exit code.
    pub fn assert_failure(&self) -> &Self {
        if self.status == 0 {
            panic!(
                "assertion failed: expected failure (non-zero exit code), got exit code 0\nstdout: {}",
                self.stdout
            );
        }
        self
    }

    /// Assert that the command exited with the given exit code.
    pub fn assert_exit_code(&self, code: i32) -> &Self {
        if self.status != code {
            panic!(
                "assertion failed: expected exit code {}, got {}\nstdout: {}\nstderr: {}",
                code, self.status, self.stdout, self.stderr
            );
        }
        self
    }

    /// Assert that stdout contains the given substring.
    pub fn assert_stdout_contains(&self, substring: &str) -> &Self {
        if !self.stdout.contains(substring) {
            panic!(
                "assertion failed: stdout does not contain {:?}\nstdout: {:?}",
                substring, self.stdout
            );
        }
        self
    }

    /// Assert that stdout equals the given string exactly.
    pub fn assert_stdout_equals(&self, expected: &str) -> &Self {
        if self.stdout != expected {
            panic!(
                "assertion failed: stdout does not equal expected\n  expected: {:?}\n  actual:   {:?}",
                expected, self.stdout
            );
        }
        self
    }

    /// Assert that stdout is empty.
    pub fn assert_stdout_is_empty(&self) -> &Self {
        if !self.stdout.is_empty() {
            panic!(
                "assertion failed: expected stdout to be empty, got: {:?}",
                self.stdout
            );
        }
        self
    }

    /// Assert that stdout has exactly the given number of non-empty lines.
    pub fn assert_stdout_line_count(&self, count: usize) -> &Self {
        let lines: Vec<&str> = self.stdout.lines().collect();
        if lines.len() != count {
            panic!(
                "assertion failed: expected {} stdout line(s), got {}\nstdout: {:?}",
                count,
                lines.len(),
                self.stdout
            );
        }
        self
    }

    /// Assert that stdout matches the given glob pattern.
    ///
    /// Supports `*` (any number of characters) and `?` (exactly one character).
    pub fn assert_stdout_matches(&self, pattern: &str) -> &Self {
        if !glob_match(pattern, self.stdout.trim()) {
            panic!(
                "assertion failed: stdout does not match pattern {:?}\nstdout: {:?}",
                pattern, self.stdout
            );
        }
        self
    }

    /// Assert that stderr contains the given substring.
    pub fn assert_stderr_contains(&self, substring: &str) -> &Self {
        if !self.stderr.contains(substring) {
            panic!(
                "assertion failed: stderr does not contain {:?}\nstderr: {:?}",
                substring, self.stderr
            );
        }
        self
    }

    /// Assert that stderr equals the given string exactly.
    pub fn assert_stderr_equals(&self, expected: &str) -> &Self {
        if self.stderr != expected {
            panic!(
                "assertion failed: stderr does not equal expected\n  expected: {:?}\n  actual:   {:?}",
                expected, self.stderr
            );
        }
        self
    }

    /// Assert that stderr is empty.
    pub fn assert_stderr_is_empty(&self) -> &Self {
        if !self.stderr.is_empty() {
            panic!(
                "assertion failed: expected stderr to be empty, got: {:?}",
                self.stderr
            );
        }
        self
    }

    /// Split stdout into lines.
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }
}

/// Simple glob matching supporting `*` (any characters) and `?` (one character).
fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_inner(&pat, &txt)
}

fn glob_match_inner(pattern: &[char], text: &[char]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    match pattern[0] {
        '*' => {
            // Try matching zero or more characters
            for i in 0..=text.len() {
                if glob_match_inner(&pattern[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if text.is_empty() {
                false
            } else {
                glob_match_inner(&pattern[1..], &text[1..])
            }
        }
        c => {
            if text.is_empty() || text[0] != c {
                false
            } else {
                glob_match_inner(&pattern[1..], &text[1..])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn echo_cmd(text: &str) -> Cmd {
        Cmd::new("cmd").arg("/C").arg(format!("echo {}", text))
    }

    #[cfg(not(windows))]
    fn echo_cmd(text: &str) -> Cmd {
        Cmd::new("echo").arg(text)
    }

    #[test]
    fn test_echo_success() {
        let output = echo_cmd("hello").run().unwrap();
        output.assert_success();
    }

    #[test]
    fn test_assert_success() {
        let output = cmd("rustc").arg("--version").run().unwrap();
        output.assert_success();
    }

    #[test]
    fn test_assert_failure() {
        let output = cmd("rustc").arg("--invalid-flag-xyz").run().unwrap();
        output.assert_failure();
    }

    #[test]
    fn test_assert_stdout_contains() {
        let output = echo_cmd("hello world").run().unwrap();
        output.assert_stdout_contains("hello");
    }

    #[test]
    fn test_assert_stdout_equals() {
        let output = echo_cmd("hello").run().unwrap();
        let trimmed = output.stdout.trim().to_string();
        let expected_output = CmdOutput {
            status: 0,
            stdout: format!("{}\n", trimmed),
            stderr: String::new(),
        };
        expected_output.assert_stdout_equals(&format!("{}\n", trimmed));
    }

    #[test]
    fn test_assert_stderr_is_empty() {
        let output = cmd("rustc").arg("--version").run().unwrap();
        output.assert_stderr_is_empty();
    }

    #[test]
    fn test_assert_exit_code() {
        let output = cmd("rustc").arg("--version").run().unwrap();
        output.assert_exit_code(0);
    }

    #[test]
    fn test_env_variable() {
        #[cfg(windows)]
        let output = cmd("cmd")
            .args(&["/C", "echo %TEST_VAR%"])
            .env("TEST_VAR", "my_value")
            .run()
            .unwrap();

        #[cfg(not(windows))]
        let output = cmd("sh")
            .args(&["-c", "echo $TEST_VAR"])
            .env("TEST_VAR", "my_value")
            .run()
            .unwrap();

        output.assert_success().assert_stdout_contains("my_value");
    }

    #[test]
    fn test_stdin_piping() {
        #[cfg(windows)]
        let output = cmd("findstr")
            .arg("hello")
            .stdin("hello world\ngoodbye\n")
            .run()
            .unwrap();

        #[cfg(not(windows))]
        let output = cmd("cat")
            .stdin("hello world\n")
            .run()
            .unwrap();

        output.assert_success().assert_stdout_contains("hello");
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("hello*", "hello world"));
        assert!(glob_match("hello*", "hello"));
        assert!(glob_match("*world", "hello world"));
        assert!(glob_match("h?llo", "hello"));
        assert!(!glob_match("h?llo", "hllo"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
        assert!(glob_match("a*b*c", "aXXbYYc"));
        assert!(!glob_match("a*b*c", "aXXbYY"));
    }

    #[test]
    fn test_stdout_lines() {
        let output = CmdOutput {
            status: 0,
            stdout: "line1\nline2\nline3".to_string(),
            stderr: String::new(),
        };
        assert_eq!(output.stdout_lines(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_assert_stdout_line_count() {
        let output = CmdOutput {
            status: 0,
            stdout: "line1\nline2\nline3".to_string(),
            stderr: String::new(),
        };
        output.assert_stdout_line_count(3);
    }

    #[test]
    fn test_assert_stdout_is_empty() {
        let output = CmdOutput {
            status: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        output.assert_stdout_is_empty();
    }

    #[test]
    fn test_assert_stdout_matches() {
        let output = echo_cmd("hello world").run().unwrap();
        output.assert_stdout_matches("hello*");
    }

    #[test]
    fn test_current_dir() {
        #[cfg(windows)]
        let output = cmd("cmd")
            .args(&["/C", "cd"])
            .current_dir("C:\\")
            .run()
            .unwrap();

        #[cfg(not(windows))]
        let output = cmd("pwd")
            .current_dir("/tmp")
            .run()
            .unwrap();

        output.assert_success();
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_assert_success_panics_on_failure() {
        let output = CmdOutput {
            status: 1,
            stdout: String::new(),
            stderr: String::new(),
        };
        output.assert_success();
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_assert_failure_panics_on_success() {
        let output = CmdOutput {
            status: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        output.assert_failure();
    }

    #[test]
    fn test_chaining() {
        let output = cmd("rustc").arg("--version").run().unwrap();
        output
            .assert_success()
            .assert_exit_code(0)
            .assert_stdout_contains("rustc")
            .assert_stderr_is_empty();
    }
}
