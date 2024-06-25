# envelope
envelope is a modern environment variables manager.

```
A modern environment variables manager

Usage: envelope [COMMAND]

Commands:
  add        Add environment variables to a specific environment
  check      Check which environment is currently exported
  delete     Delete environment variables
  drop       Drop environment
  duplicate  Create a copy of another environment
  export     Export environment variables
  edit       Edit environment variables in editor
  init       Initialize envelope
  import     Import environment variables
  list       List saved environments and/or their variables
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Installation

### Binary
You can download the envelope binary in the latest
[release](https://github.com/mattrighetti/envelope/releases/latest) and copy the
binary to a folder in your `$PATH`

### Cargo
You can install envelope with cargo, make sure that your `~/.cargo` folder is in
your `$PATH`
```sh
$ git clone https://github.com/mattrighetti/envelope
$ cd envelope
$ cargo install --path .
$ envelope --version
envelope 0.3.11
```

## Building
envelope is written in Rust, so you'll need the [Rust
compiler](https://www.rust-lang.org/).

To build envelope:
```sh
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

## Usage

### Pretty print
Pipe .env files to envelope to get a pretty format representation of the file
```
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

```
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
```
$ cat .env | envelope import prod
```

### List
List env variables of a particular enviroment
```
$ envelope list dev
API_KEY=your_api_key
...
SMTP_HOST=smtp.example.com
```

### Export
Export environment variables to a .env file in current directory
```
$ envelope export prod
```
This will create a .env file containing all the variables that you have stored
in your `prod` enviroment in envelope.

This makes it easy to switch between different .env configurations, need to use the
prod envs? Just run `envelope export prod`, want to switch to your dev ones? Run
`envelope export dev` and everything will be handled for you, for free.

You can also output to a specific file with the `-o` flag:
```
$ envelope export prod -o .env.prod
```

### Add
Add env variables to an environment
```
$ envelope add local db_connection https://example.com
$ envelope list local
DB_CONNECTION=https://examples.com
```
You can use lowercased variables, they will be uppercased by envelope

### Delete
Delete entire environments from envelope
```
$ envelope delete dev
$ envelope list dev
```
Envelope always soft deletes environment variables, they are never actually
deleted, this is useful in case you want to take a look at the history of a
certain valriable. You can however do a hard delete using the `drop` command

### Drop
Drops (hard deletes) an environment
```sh
$ envelope drop dev
$ envelope list
```

### Check
Checks which environment is currently active
```sh
$ export $(envelope list dev)
$ envelope check
dev
```

