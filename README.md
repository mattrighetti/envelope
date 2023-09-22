# envelope
envelope is a command line utility and tool to help you manage your .env files.

## How it works
envelope is basically a command line utility that leverages an SQLite database
to keep track of your enviroment variables so that you can easily switch between
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
$ envelope list
+-------------+-------------------+----------------------------------------------+
| ENVIRONMENT | VARIABLE          | VALUE                                        |
+-------------+-------------------+----------------------------------------------+
| dev         | API_KEY           | your_api_key_here                            |
+-------------+-------------------+----------------------------------------------+
| dev         | AWS_ACCESS_KEY_ID | your_access_key_id                           |
+-------------+-------------------+----------------------------------------------+
| dev         | DATABASE_URL      | postgres://user:password@localhost:5432/mydb |
+-------------+-------------------+----------------------------------------------+
| dev         | DEBUG_MODE        | true                                         |
+-------------+-------------------+----------------------------------------------+
| dev         | SECRET_KEY        | mysecretkey123                               |
+-------------+-------------------+----------------------------------------------+
| dev         | SMTP_HOST         | smtp.example.com                             |
+-------------+-------------------+----------------------------------------------+
```

It's also possible to import directly from stdin
```
$ cat .env | envelope import prod
```

### List
List env variables of a particular enviroment
```
$ envelope list dev
+-------------+-------------------+----------------------------------------------+
| ENVIRONMENT | VARIABLE          | VALUE                                        |
+-------------+-------------------+----------------------------------------------+
| dev         | API_KEY           | your_api_key_here                            |
+-------------+-------------------+----------------------------------------------+
+   ...       +      ...          +                   .......                    +
+-------------+-------------------+----------------------------------------------+
| dev         | SMTP_HOST         | smtp.example.com                             |
+-------------+-------------------+----------------------------------------------+
```

### Export
Create a .env file with variables of a specific enviroment
```
$ envelope export prod
```
This will create a .env file containing all the variables that you have stored
in your `prod` enviroment in envelope.

This makes it easy to switch between different configurations, need to use the
prod envs? Just run `envelope export prod`, want to switch to your dev ones? Run
`envelope export dev` and everything will be handled for you, for free.

You can also output to a specific file with the `-o` flag:
```
$ envelope export prod -o .env.prod
```

### Add
Add env variables
```
$ envelope add local DB_CONNECTION https://example.com
$ envelope list local
+-------------+-------------------+-----------------------+
| ENVIRONMENT | VARIABLE          | VALUE                 |
+-------------+-------------------+-----------------------+
| local       | DB_CONNECTION     | https://examples.com  |
+-------------+-------------------+-----------------------+
```

### Delete
Delete entire environments from envelope
```
$ envelope delete dev
$ envelope list dev
```
