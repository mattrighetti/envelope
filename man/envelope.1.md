% envelope(1) v0.7.1210

NAME
====
envelope — a modern environment variables manager

SYNOPSIS
========
`envelope [command]`

*envelope* is a modern environment variables manager.

COMMANDS
========
- **add**: Add environment variables to a specific environment.
- **check**: Check which environment is currently exported.
- **delete**: Delete environment variables.
- **drop**: Drop an environment.
- **duplicate**: Create a copy of another environment.
- **diff**: Diff two existing environments.
- **edit**: Edit environment variables in the editor.
- **history**: Display the historical values of a specific key in a given environment.
- **init**: Initialize envelope.
- **import**: Import environment variables.
- **list**: List saved environments and/or their variables.
- **revert**: Revert environment variables to a previous state.
- **help**: Print this message or the help of the given subcommand(s).

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
Lists all environment variables in the 'dev' environment using the default `kv` format (`KEY=value`).

```bash
envelope list dev --shell sh
```
Lists all environment variables in the 'dev' environment as `export KEY='value'` commands compatible with POSIX-compliant shells (`sh`, `bash`, `zsh`).

```bash
envelope list dev --shell fish
```
Lists all environment variables in the 'dev' environment as `set -gx KEY 'value'` commands for Fish.

```bash
envelope list dev --shell nu
```
Lists all environment variables in the 'dev' environment as a Nushell record suitable for `load-env`.

```bash
envelope list dev --shell cmd
```
Lists all environment variables in the 'dev' environment as `set "KEY=value"` commands for Windows Command Prompt.

```bash
envelope list dev --shell powershell
```
Lists all environment variables in the 'dev' environment as `$env:KEY = "value"` commands for PowerShell.

`--shell` supports: `kv`, `sh`, `fish`, `nu`, `cmd`, `powershell`.
Aliases: `bash`/`zsh` -> `sh`, `nushell` -> `nu`, `pwsh` -> `powershell`.

```bash
envelope duplicate dev dev-local
```
Creates a new 'dev-local' environment with the same variables stored in 'dev'.

```bash
envelope check
```
Returns all the environments that are active by comparing active environment variables in the current process.

```bash
envelope edit dev-local
```
Edit variables of 'dev-local' in the default editor. If you want to specify a different editor, you can do so by using the `ENVELOPE_EDITOR` environment variable.

```bash
envelope drop dev-local
```
Hard delete from the database every environment variable stored in 'dev-local'.

```bash
envelope add dev-local KEY VALUE
```
Adds environment variable `KEY=VALUE` in 'dev-local'.

```bash
envelope delete dev-local KEY
```
Deletes environment variable `KEY` in 'dev-local'.

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
