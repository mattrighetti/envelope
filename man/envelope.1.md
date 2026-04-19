% envelope(1) v0.8.0210

NAME
====
envelope — a modern environment variables manager

SYNOPSIS
========
`envelope [COMMAND] [OPTIONS]`

DESCRIPTION
===========
*envelope* stores environment variables in a local SQLite database, organized
into named environments. Variables can be imported from `.env` files, edited
interactively, diffed, and tracked with full history. The database can be
encrypted at rest and commands can inject variables directly into subprocesses.

TERMINOLOGY
===========

**environment** (or *env*)
:   A named collection of environment variables, like "dev", "prod", or "staging".
    Stored in the `.envelope` database.

**key**
:   The name of a variable, e.g., `DATABASE_URL`, `API_KEY`, `DEBUG`.

**value**
:   The value assigned to a key, e.g., `postgres://localhost:5432`, `sk-1234abcd`, `true`.

**key-value pair** or **variable**
:   A key and its value together, represented as `KEY=VALUE`.

COMMANDS
========

**init**
:   Initialize envelope in the current directory. Creates the `.envelope`
    SQLite database used to store all environments and variables.

**add** *env* *key* [*value*] [`--stdin`]
:   Add or update variable *key* in environment *env*. If *value* is omitted
    the variable is set to an empty string.

    `--stdin`  Read the value from stdin (useful for secrets — avoids shell history).

**check**
:   Compare the current shell's environment against all stored environments
    and report which ones are active.

**delete** [`--env` *env*] [`--key` *key*]
:   Soft-delete a variable or environment (marks as deleted but preserves history).
    Behavior depends on which flags are given:

    - `--env` *env* `--key` *key*  Delete variable *key* from *env* only.
    - `--key` *key*                Delete variable *key* from every environment.
    - `--env` *env*                Delete the entire environment *env*.

**diff** *env1* *env2*
:   Compare two environments. Variables only in *env1* are shown in green,
    only in *env2* in red, and variables present in both with different values
    in gray.

**drop** *env*
:   Hard-delete environment *env* and all its variables permanently (cannot be recovered).

**duplicate** *source* *target*
:   Create a new environment *target* as a copy of *source*.

**edit** *env*
:   Open environment *env* in your editor for interactive editing.
    The editor is chosen from `ENVELOPE_EDITOR`, falling back to `EDITOR`.

**history** *env* *key*
:   Show all past values of variable *key* in environment *env*, newest first.

**import** *env* [*file*]
:   Import variables from a `.env`-formatted *file* into environment *env*.
    Reads from stdin if *file* is not provided.

**list** [*env*] [`-p`] [`-t`] [`-s` *order*]
:   Without *env*, list all environment names. With *env*, list its variables.

    `-p`, `--pretty-print`  Display variables in a formatted table.
    `-t`, `--truncate`      Truncate long values in table output (implies `-p`).
    `-s`, `--sort` *order*  Sort order: `k` key asc, `kd` key desc, `v` value asc,
                            `vd` value desc, `d` date asc (default), `dd` date desc.

**lock**
:   Encrypt the database with a password. Once locked, every command that reads
    or writes data will prompt for the password.

**revert** *env* *key*
:   Roll back variable *key* in environment *env* to its previous value.

**run** [`-i`] *env* `--` *command* [*args*...]
:   Execute *command* with the variables from environment *env* injected.

    `-i`, `--isolated`  Do not inherit variables from the parent shell —
                        only the stored variables are visible to the command.

**unlock**
:   Decrypt the database so that subsequent commands run without a password prompt.

EXAMPLES
========
```bash
cat .env | envelope
```
Pretty prints environment variables in the `.env` file.

```bash
envelope init
```
Creates an `.envelope` file in the current directory, needed by envelope to store your environment variables.

```bash
envelope import dev .env
```
Imports variables from `.env` file into the environment named 'dev'.

```bash
envelope list
```
Lists all environments.

```bash
envelope list dev
```
Lists all environment variables in the 'dev' environment.

```bash
envelope list -p -s kd dev
```
Lists variables in 'dev' in a formatted table, sorted by key descending.

```bash
envelope duplicate dev dev-local
```
Creates a new 'dev-local' environment with the same variables stored in 'dev'.

```bash
envelope run dev -- bash
# (now in a subshell with 'dev' environment variables)
envelope check
```
Reports which stored environments are currently active. Useful to verify which
environment's variables you're working with. In the example, `check` would show
that the 'dev' environment is active in the current shell.

```bash
envelope edit dev-local
```
Edit variables of 'dev-local' in the default editor. If you want to specify a different editor, you can do so by using the `ENVELOPE_EDITOR` environment variable.

```bash
envelope drop dev-local
```
Permanently remove environment 'dev-local' and all its variables from the database.
Unlike `delete`, this cannot be undone.

```bash
envelope add dev-local KEY VALUE
```
Adds environment variable `KEY=VALUE` in 'dev-local'.

```bash
envelope add dev-local SECRET_KEY --stdin
```
Prompts for the value of `SECRET_KEY` via stdin, keeping it out of shell history.

```bash
envelope delete --env dev-local --key KEY
```
Soft-delete environment variable `KEY` in 'dev-local' (history is preserved and can be reverted).

```bash
envelope delete --key KEY
```
Deletes environment variable `KEY` from every environment.

```bash
envelope diff env1 env2
```
Compares two environments ('env1' and 'env2') and displays the differences between their variables.

```bash
envelope history dev-local KEY
```
Displays the historical values of the environment variable `KEY` in 'dev-local'.

```bash
envelope revert dev-local KEY
```
Reverts the environment variable `KEY` in 'dev-local' to its previous state.

```bash
envelope run dev -- node server.js
```
Runs `node server.js` with environment variables from the 'dev' environment injected into the process.

```bash
envelope run --isolated dev -- node server.js
```
Same as above, but the command does not inherit any variables from the parent shell — only those stored in 'dev' are available.

```bash
envelope lock
```
Encrypts the envelope database with a password. After locking, a password is required to run any command.

```bash
envelope unlock
```
Decrypts the envelope database. After unlocking, commands run without prompting for a password.

EXIT STATUSES
=============
- **0**: If everything goes OK.
- **1**: If there was an I/O error during operation.
- **2**: If there was a problem with the command-line arguments.

AUTHOR
======
envelope is maintained by Mattia Righetti.

**Website:** <https://mattrighetti.com/>

**Source code:** <https://github.com/mattrighetti/envelope>
