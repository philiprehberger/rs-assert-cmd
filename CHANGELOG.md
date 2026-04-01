# Changelog

## 0.1.4 (2026-03-31)

- Standardize README to 3-badge format with emoji Support section
- Update CI checkout action to v5 for Node.js 24 compatibility

## 0.1.3 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

## 0.1.2 (2026-03-22)

- Fix README, CHANGELOG, and CI compliance

## 0.1.1 (2026-03-20)

- Re-release with registry token configured

## 0.1.0 (2026-03-19)

- Fluent `Cmd` builder for spawning CLI processes
- Builder methods: `arg`, `args`, `env`, `stdin`, `current_dir`, `timeout`
- `CmdOutput` with chainable assertion methods
- `assert_success` / `assert_failure` for exit code checks
- `assert_exit_code` for specific exit code matching
- `assert_stdout_contains`, `assert_stdout_equals`, `assert_stdout_is_empty`
- `assert_stdout_line_count` and `stdout_lines` helper
- `assert_stdout_matches` with simple glob pattern support (`*` and `?`)
- `assert_stderr_contains`, `assert_stderr_equals`, `assert_stderr_is_empty`
- Descriptive panic messages on assertion failures
- Zero dependencies — uses only `std::process::Command`
