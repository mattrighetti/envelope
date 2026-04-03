# E2E Tests

This directory contains the end-to-end test spec for the public `envelope` CLI.

The runner lives at [ci/e2e-test](/Users/matt/Developer/envelope/ci/e2e-test). It loads every `*.toml` file in this directory and then runs each scenario in filename order.

## Layout

- `listing.toml`, `run.toml`, `import.toml`, etc.: one scenario per file

Each scenario file should define exactly one `[[scenario]]` block. Scenario setup should stay in the same file so each test is readable in isolation.

## Running The Tests

Build the release binary first:

```sh
cargo build --release
```

Run the full e2e suite:

```sh
ci/e2e-test target/release/envelope
```

Run a single scenario or a filtered subset:

```sh
ci/e2e-test target/release/envelope listing
ci/e2e-test target/release/envelope run
```

The filter matches either the scenario name or an individual case label.

## Spec Format

Scenario setup is declared in the same file:

```toml
[[scenario.setup]]
commands = [
  ["init"],
  ["add", "dev", "DATABASE_URL", "postgres://localhost/dev"],
  ["add", "dev", "SECRET_KEY", "abc123"],
]
```

Scenarios then declare their assertions as individual cases:

```toml
[[scenario]]
name = "listing"

[[scenario.setup]]
commands = [
  ["init"],
  ["add", "dev", "DATABASE_URL", "postgres://localhost/dev"],
  ["add", "dev", "SECRET_KEY", "abc123"],
]

[[scenario.case]]
label = "list dev"
command = ["list", "dev"]
stdout = """
DATABASE_URL=postgres://localhost/dev
SECRET_KEY=abc123
"""
```

## Supported Fields

Setup entries support:

- `commands`: compact list of setup commands
- `sleep_ms`: pause before the next step

Cases support:

- `label`: human-readable case name
- `env`: environment variables in `KEY=value` form
- `stdin`: text passed to the command on stdin
- `command`: command arguments passed to `envelope`
- `status`: expected exit code, default `0`
- `stdout`: exact expected stdout
- `stderr`: exact expected stderr

## Notes

- Each scenario runs in its own temporary working directory, so files are isolated by default.
- Prepared fixture files can live directly in [ci/e2e](/Users/matt/Developer/envelope/ci/e2e) and be referenced from cases using paths relative to `tmp/e2e.*`.
- Case assertions are exact. If command output changes, update the expected `stdout` or `stderr` text in the scenario file.
- The runner creates a temporary working directory under `tmp/` and removes it when the run finishes.
