# envelope
envelope is a modern environment variables manager.

```console
A modern environment variables manager

Usage: envelope [COMMAND]

Commands:
  add        Add environment variables to a specific environment
  check      Check which environment is currently exported
  delete     Delete environment variables
  drop       Drop environment
  duplicate  Create a copy of another environment
  diff       Diff two existing environments
  edit       Edit environment variables in editor
  history    Display the historical values of a specific key in a given environment
  init       Initialize envelope
  import     Import environment variables
  list       List saved environments and/or their variables
  lock       Encrypt envelope
  revert     Revert environment variable
  run        Run a command with environment variables from a specific environment
  unlock     Decrypt the envelope
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Installation

### Brew
You can install envelope from homebrew-core:
```console
$ brew install envelope
```

### Binary
You can download the envelope binary in the latest
[release](https://github.com/mattrighetti/envelope/releases/latest) and copy the
binary to a folder in your `$PATH`

### Cargo
You can install envelope with cargo, make sure that your `~/.cargo` folder is in
your `$PATH`
```console
$ cargo install --git https://github.com/mattrighetti/envelope
$ envelope --version
envelope 0.3.11
```

## Building
envelope is written in Rust, so you'll need the [Rust
compiler](https://www.rust-lang.org/).

To build envelope:
```console
$ git clone https://github.com/mattrighetti/envelope
$ cd envelope
$ cargo build --release
$ ./target/release/envelope --version
envelope 0.3.11
```

## How it works
`envelope` is a command line utility that leverages an SQLite database
to keep track of your environment variables so you can easily switch between
different configurations.

## Quick Start

Initialize envelope in your project directory:
```console
$ cd my-project
$ envelope init
```

Import your existing `.env` file:
```console
$ envelope import dev .env
```

Export variables to your shell:
```console
$ export $(envelope list dev)
```

Verify which environment is active:
```console
$ envelope check
dev
```

## Usage

### Init
Initialize envelope in your current directory. This creates a `.envelope` database file.
```console
$ envelope init
```

> [!NOTE]
> You must run `envelope init` before using any other commands. The `.envelope` file
> should be added to your `.gitignore` to avoid committing sensitive environment variables.

### Pretty print
Pipe .env files to envelope to get a pretty format representation of the file
```console
$ cat .env | envelope

+-------------------+----------------------------------------------+
| VARIABLE          | VALUE                                        |
+-------------------+----------------------------------------------+
| DATABASE_URL      | postgres://user:password@localhost:5432/mydb |
+-------------------+----------------------------------------------+
| SECRET_KEY        | mysecretkey123                               |
+-------------------+----------------------------------------------+
| API_KEY           | your_api_key_here                            |
+-------------------+----------------------------------------------+
| DEBUG_MODE        | true                                         |
+-------------------+----------------------------------------------+
| SMTP_HOST         | smtp.example.com                             |
+-------------------+----------------------------------------------+
| AWS_ACCESS_KEY_ID | your_access_key_id                           |
+-------------------+----------------------------------------------+
```

### Import
Import from .env file

```console
$ envelope import dev .env
$ envelope list dev
API_KEY=your_api_key_here
AWS_ACCESS_KEY_ID=your_access_key_id
DATABASE_URL=postgres://user:password@localhost:5432/mydb
DEBUG_MODE=true
SECRET_KEY=mysecretkey123
SMTP_HOST=smtp.example.com
```

It's also possible to import directly from stdin
```console
$ cat .env | envelope import prod
```

### List
List all saved environments:
```console
$ envelope list
dev
staging
prod
```

List environment variables of a particular environment:
```console
$ envelope list dev
API_KEY=your_api_key
DATABASE_URL=postgres://user:password@localhost:5432/mydb
DEBUG_MODE=true
SECRET_KEY=mysecretkey123
SMTP_HOST=smtp.example.com
```

Pretty print with a table format:
```console
$ envelope list dev --pretty-print
+-------------+----------------------------------------------+-------+
| ENVIRONMENT | VARIABLE                                     | VALUE |
+-------------+----------------------------------------------+-------+
| dev         | DATABASE_URL                                 | pos.. |
+-------------+----------------------------------------------+-------+
| dev         | SECRET_KEY                                   | mys.. |
+-------------+----------------------------------------------+-------+
...
```

Truncate long values in pretty print mode:
```console
$ envelope list dev --pretty-print --truncate
```

**Sorting Options**

You can specify the sorting order using the `--sort` option:

- `key` or `k`: Sort by key in ascending order
- `value` or `v`: Sort by value in ascending order
- `date` or `d`: Sort by creation date in ascending order
- `kd`: Sort by key in descending order
- `vd`: Sort by value in descending order
- `dd`: Sort by creation date in descending order (default)

Example:
```console
$ envelope list dev --sort kd
SMTP_HOST=smtp.example.com
SECRET_KEY=mysecretkey123
DEBUG_MODE=true
DATABASE_URL=postgres://user:password@localhost:5432/mydb
API_KEY=your_api_key
```

### Add
Add environment variables to an environment:
```console
$ envelope add dev api_key sk_test_123456789
$ envelope list dev
API_KEY=sk_test_123456789
```

Variable names are automatically uppercased:
```console
$ envelope add dev database_url postgres://localhost/mydb
$ envelope list dev
DATABASE_URL=postgres://localhost/mydb
```

Read value from stdin for sensitive data:
```console
$ envelope add dev secret_token --stdin
Enter value for env secret_token:
my-super-secret-token
```

Add a variable with an empty value:
```console
$ envelope add dev optional_var
$ envelope list dev
OPTIONAL_VAR=
```

### Delete
Delete a specific variable from an environment:
```console
$ envelope delete --env dev --key API_KEY
$ envelope list dev
# API_KEY will no longer appear
```

Delete a variable across all environments:
```console
$ envelope delete --key DEBUG_MODE
# Removes DEBUG_MODE from all environments where it exists
```

Delete an entire environment:
```console
$ envelope delete --env dev
$ envelope list
# dev will no longer appear
```

> [!NOTE]
> Envelope always **soft deletes** environment variables. They are never actually
> removed from the database, which allows you to view the history and revert changes.
> For a hard delete that permanently removes data, use the `drop` command.

### Drop
Drops (hard deletes) an environment and permanently removes all its variables from the database:
```console
$ envelope drop dev
$ envelope list
# dev is permanently deleted, including all its history
```

> [!WARNING]
> Unlike `delete`, the `drop` command permanently removes all data.
> This cannot be undone and you will not be able to view history or revert changes.

### Duplicate
Create a copy of an existing environment:
```console
$ envelope add dev api_key sk_dev_12345
$ envelope add dev database_url postgres://localhost/devdb
$ envelope duplicate dev staging
$ envelope list staging
API_KEY=sk_dev_12345
DATABASE_URL=postgres://localhost/devdb
```

This is useful for:
- Creating a new environment based on an existing one
- Copying production settings to staging for testing
- Creating backups before making changes

Example workflow:
```console
# Create a backup before experimenting
$ envelope duplicate prod prod-backup

# Make changes to prod
$ envelope add prod api_key sk_new_key

# If something goes wrong, restore from backup
$ envelope duplicate prod-backup prod
```

### Edit
Open your `$EDITOR` to interactively edit environment variables:
```console
$ envelope edit dev
```

This opens your default editor with all variables in the format:
```
API_KEY=your_api_key
DATABASE_URL=postgres://localhost/mydb
DEBUG_MODE=true
```

**Editing tips:**
- Modify values by changing the text after `=`
- Delete a variable by commenting it out with `#`:
  ```
  #API_KEY=your_api_key
  ```
- Add new variables by adding new lines:
  ```
  NEW_VAR=new_value
  ```

When you save and close the editor, envelope will:
- Update all modified values
- Delete all commented variables (soft delete)
- Add all new variables

### Check
Check which environment(s) are currently exported in your shell:
```console
$ export $(envelope list dev)
$ envelope check
dev
```

Check works by comparing your current shell's environment variables against
all stored environments. It will show all environments whose variables exactly
match what's currently exported.

Multiple environments can match if they share the same variables:
```console
$ envelope add shared-config api_key sk_12345
$ envelope duplicate shared-config also-shared
$ export $(envelope list shared-config)
$ envelope check
shared-config
also-shared
```

If no environment matches completely, nothing is returned:
```console
$ envelope check
# No output means no environment is fully active
```

### Lock
Encrypt the envelope database. You will be prompted for a password and a
confirmation.
```console
$ envelope lock
Password: ********
Confirm password: ********
database locked successfully
```

> [!NOTE]
>
> when the database is locked, all other commands
> will fail until you run `envelope unlock`.

### Unlock
Decrypt the envelope database with the password you set when locking it.
```console
$ envelope unlock
Password: ********
database unlocked successfully
```

### Diff
Compare two environments to see their differences:
```diff
$ envelope add local db_connection http://localhost:3030
$ envelope add local dev true
$ envelope add prod db_connection https://proddb.com
$ envelope add prod db_user pg
$ envelope add prod db_pwd somepwd
$ envelope diff local prod
# DB_CONNECTION=http://localhost:3030 -> https://proddb.com
- DB_PWD=somepwd
- DB_USER=pg
+ DEV=true
```

The output shows:
- **Gray** `#` lines: Variables present in both environments but with different values
  - First value is from the source environment (local)
  - Second value is from the target environment (prod)
- **Red** `-` lines: Variables only in the target environment (prod)
- **Green** `+` lines: Variables only in the source environment (local)

**Use cases:**
```console
# Compare development and production configs
$ envelope diff dev prod

# Verify staging matches production
$ envelope diff staging prod

# Check what changed before deploying
$ envelope diff current-prod new-prod
```

### Revert
Revert a variable to its previous value. Each call to `revert` moves one step back in history:
```console
$ envelope add local db_connection http://localhost:3030
$ envelope add local db_connection http://localhost:2222
$ envelope add local db_connection http://localhost:3333
$ envelope list local
DB_CONNECTION=http://localhost:3333

$ envelope revert local db_connection
$ envelope list local
DB_CONNECTION=http://localhost:2222

$ envelope revert local db_connection
$ envelope list local
DB_CONNECTION=http://localhost:3030
```

**Practical example:**
```console
# Accidentally set wrong API key
$ envelope add prod api_key sk_wrong_key_12345
# Oops! Revert to the previous value
$ envelope revert prod api_key
# Back to the correct key
```

> [!NOTE]
> Revert works because envelope keeps a complete history of all changes.
> You can revert multiple times to go back through the entire history.
> Use `envelope history` to see all historical values before reverting.

### Run
Run a command with environment variables from a specific environment
automatically injected. This avoids the need to manually export variables into
your shell.
```console
$ envelope run dev -- cargo run
```

You can run any command and its arguments. The -- separator is optional but
recommended when your command includes its own flags.

If you wish to clear the inherited envs from the parent process, you can use the
`--isolated` flag and that will make sure only the selected env variables are
injected in the subprocess.

```console
$ envelope add prod ENVIRONMENT=prod
$ envelope run prod env
PATH=...
HOME=/Users/matt
SHELL=/bin/zsh
ENVIRONMENT=prod

$ envelope run prod --isolated env
# only PATH is inherited from parent
PATH=...
ENVIRONMENT=prod
```

**Use cases**:
- Run scripts without polluting your current shell environment.
- Quickly test different environments against the same command.
- Use in CI/CD pipelines to wrap execution steps.

### History
View the complete history of a variable, including deleted values:
```console
$ envelope add local db_connection http://localhost:3030
$ envelope add local db_connection http://localhost:2222
$ envelope add local db_connection http://localhost:3333
$ envelope history local db_connection
2025-01-01 00:00:00 DB_CONNECTION=http://localhost:3030
2025-02-01 00:00:00 DB_CONNECTION=http://localhost:2222
2025-03-01 00:00:00 DB_CONNECTION=http://localhost:3333
```

View deleted variables:
```console
$ envelope add dev api_key sk_old_key
$ envelope add dev api_key sk_new_key
$ envelope delete --env dev --key api_key
$ envelope history dev api_key
2025-01-01 10:00:00 API_KEY=sk_old_key
2025-01-01 11:00:00 API_KEY=sk_new_key
```

**Use cases:**
- Audit when and how a variable changed
- Find the previous value before a mistake
- Track configuration changes over time
- Recover deleted variables by seeing their last value

## Common Workflows

### Starting a New Project
```console
# Initialize envelope in your project
$ cd my-project
$ envelope init

# Import existing .env files
$ envelope import dev .env.development
$ envelope import prod .env.production

# Add .envelope to .gitignore
$ echo ".envelope" >> .gitignore
```

### Daily Development
```console
# Switch between environments
$ export $(envelope list dev)
$ npm start

# Check what's currently active
$ envelope check
dev

# Switch to staging
$ export $(envelope list staging)
$ envelope check
staging
```

### Managing Secrets Securely
```console
# Add sensitive values without exposing them in shell history
$ envelope add prod database_password --stdin
Enter value for env database_password:
[type your password]

# Lock the database when not in use
$ envelope lock
Password: ********
Confirm password: ********

# Unlock when needed
$ envelope unlock
Password: ********
```

### Setting Up New Environments
```console
# Create staging from production
$ envelope duplicate prod staging

# Update staging-specific values
$ envelope edit staging
# Change database URLs, API endpoints, etc.

# Verify the differences
$ envelope diff staging prod
```

### Configuration Updates
```console
# Before updating production
$ envelope duplicate prod prod-backup

# Make changes
$ envelope add prod api_endpoint https://api.example.com/v2

# Verify changes
$ envelope diff prod prod-backup

# If something goes wrong
$ envelope revert prod api_endpoint
# Or restore completely
$ envelope duplicate prod-backup prod
```

### Team Collaboration
```console
# Share environment structure (not values) with team
$ envelope list dev --pretty-print > env-structure.txt

# Team member sets up their own values
$ envelope init
$ envelope add dev database_url postgres://localhost/mydb
$ envelope add dev api_key [their-own-key]

# Or import from a shared template
$ envelope import dev .env.template
```

### Debugging Environment Issues
```console
# Check if an environment is active
$ envelope check

# View current values
$ envelope list prod --pretty-print

# Compare with what's expected
$ envelope diff prod staging

# Check history of a suspicious variable
$ envelope history prod api_endpoint

# Revert if needed
$ envelope revert prod api_endpoint
```

## Tips and Tricks

### Shell Aliases
Add these to your `.bashrc` or `.zshrc` for faster workflows:
```bash
# Quick environment switching
alias envdev='export $(envelope list dev)'
alias envprod='export $(envelope list prod)'
alias envstaging='export $(envelope list staging)'

# Quick check
alias envcheck='envelope check'

# Quick edit
alias envedit='envelope edit'
```

### Using with Docker
```console
# Export directly to docker run
$ docker run --env-file <(envelope list prod) myapp

# Or save to a file
$ envelope list prod > .env.prod
$ docker run --env-file .env.prod myapp
```

### Using with Docker Compose
```yaml
# docker-compose.yml
services:
  app:
    build: .
    env_file:
      - .env.generated

# Generate .env.generated before running docker-compose
$ envelope list dev > .env.generated
$ docker-compose up
```

### CI/CD Integration
```console
# In your CI/CD pipeline, you can import secrets from your CI environment
$ envelope init
$ envelope add ci DATABASE_URL "$DATABASE_URL"
$ envelope add ci API_KEY "$API_KEY"
$ export $(envelope list ci)
$ npm test
```

### Bulk Operations
```console
# Quickly copy multiple environments
$ for env in dev1 dev2 dev3; do
    envelope duplicate prod $env
  done

# List all environments with their variable counts
$ for env in $(envelope list); do
    echo "$env: $(envelope list $env | wc -l) variables"
  done
```

### Migrating from .env Files
```console
# Import all .env.* files
$ for file in .env.*; do
    env_name=${file#.env.}
    envelope import $env_name $file
  done

# Verify imports
$ envelope list
```

### Exporting Back to .env Files
```console
# Export an environment to a .env file
$ envelope list prod > .env.production

# Export all environments
$ for env in $(envelope list); do
    envelope list $env > .env.$env
  done
```

## Security Best Practices

### Protecting Your Environment Variables

**Always add `.envelope` to `.gitignore`:**
```console
$ echo ".envelope" >> .gitignore
```

The `.envelope` file is a SQLite database containing all your environment variables.
This file should **never** be committed to version control as it contains sensitive data like:
- API keys and tokens
- Database passwords
- Secret keys
- OAuth credentials

**Use encryption for sensitive projects:**
```console
# Lock the database when not actively developing
$ envelope lock
Password: ********

# The database is now encrypted and cannot be read without the password
$ envelope list dev
error: envelope is locked - run `envelope unlock` first

# Unlock when needed
$ envelope unlock
Password: ********
```

**Recommendations:**
- Use `--stdin` flag when adding sensitive values to avoid shell history:
  ```console
  $ envelope add prod secret_key --stdin
  ```
- Use `envelope lock` on shared machines or when committing other changes
- Keep separate environments for development, staging, and production
- Regularly audit your variables using `envelope history`
- Use `envelope diff` to ensure production configs don't leak into development

### What Gets Stored

Envelope stores in the `.envelope` database:
- All environment variable keys and values
- Complete history of changes (even deleted values)
- Timestamps for all modifications

What's NOT stored:
- Your shell environment
- Files outside the current directory
- Git history or commits
