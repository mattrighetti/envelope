% envelope (1) v0.3.0

<!-- This is the envelope(1) man page, written in Markdown. -->
<!-- To generate the roff version, run `just man`, -->
<!-- and the man page will appear in the ‘target’ directory. -->

NAME
====

envelope — a modern environment variable manager

SYNOPSIS
========

`envelope [command]`

*envelope* is a modern environment variable manager.

EXAMPLES
========

`cat .env | envelope`
: Pretty prints environment variables in the .env file

`envelope init`
: Creates and `.envelope` file in the current directory, needed by envelope to
store your enviroment variables

`envelope import dev .env`
: Imports variables from .env in the tool

`envelope list`
: Lists all enviroments in the tool

`envelope list dev`
: Lists all enviroment variables in the dev enviroment

`envelope duplicate dev dev-local`
: Copies all the enviroment variables in environment dev to new environment
dev-local

`envelope check`
: Returns all the environments that are active by comparing active enviroment
varibles in the current process
