# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## 0.3.12 - 2025-02-08
#### Bug Fixes
- misc fixes and refactoring - (d5ac1ad) - Mattia Righetti
#### Miscellaneous Chores
- **(license)** update unlicense file and license field info - (147e59a) - Rui Chen
- bump ci to rust 1.84 - (51ad45a) - Mattia Righetti
- update readme - (c86e3f8) - Mattia Righetti
- add unlicense and mit license - (775b222) - Mattia Righetti
- update readme - (87319f5) - Mattia Righetti

- - -

## 0.3.11 - 2024-06-23
#### Continuous Integration
- add new builds for aarch and arm musleabihf - (7ef2850) - Mattia Righetti
#### Miscellaneous Chores
- update readme - (df19a46) - Mattia Righetti
- update readme with installation and build - (42e717e) - Mattia Righetti
- remove install section in readme temp - (d25a79f) - Mattia Righetti

- - -

## 0.3.10 - 2024-06-22
#### Bug Fixes
- add edit command description - (51e704d) - Mattia Righetti
#### Miscellaneous Chores
- update readme - (cc44a2b) - Mattia Righetti
- update readme - (5f18c40) - Mattia Righetti

- - -

## 0.3.9 - 2024-06-22
#### Bug Fixes
- list will only show latest not null values in env - (a987b7b) - Mattia Righetti
- duplicate will only duplicate latest not null values in env - (a855ea2) - Mattia Righetti
- delete ops respect soft delete contract - (04a5053) - Mattia Righetti
- delete_env now respects delete contract - (0fd7d3d) - Mattia Righetti
#### Documentation
- more db.rs function docs - (593f782) - Mattia Righetti
#### Miscellaneous Chores
- remove useless file - (a19d574) - Mattia Righetti

- - -

## 0.3.8 - 2024-06-21
#### Bug Fixes
- return error if user is trying to list non-existent variable - (d5e2576) - Mattia Righetti
- output errors on stderr - (9489c4b) - Mattia Righetti
#### Features
- impl check_env_exists to check if env is present in db - (d653f18) - Mattia Righetti

- - -

## 0.3.7 - 2024-06-21
#### Bug Fixes
- vim default editor - (91d2893) - Mattia Righetti
#### Documentation
- misc docs updates - (9a2562a) - Mattia Righetti

- - -

## 0.3.6 - 2024-06-02
#### Bug Fixes
- re-open file after editor is done with it - (c93c0ca) - Mattia Righetti
- create file in current dir and remove it afterwards - (27dba78) - Mattia Righetti

- - -

## 0.3.5 - 2024-05-19
#### Bug Fixes
- remove whitespace from value when adding variable - (2564e1f) - Mattia Righetti
#### Miscellaneous Chores
- update README - (bb9fd0b) - Mattia Righetti

- - -

## 0.3.4 - 2024-04-28
#### Bug Fixes
- remove sync command - (cba997c) - Mattia Righetti
#### Miscellaneous Chores
- **(deps)** add sea-query + binder - (4053f47) - Mattia Righetti
#### Refactoring
- moved raw sql strings to sea-query - (09a851f) - Mattia Righetti

- - -

## 0.3.3 - 2024-03-08
#### Bug Fixes
- add man page to staging step - (1359ebe) - Mattia Righetti
#### Continuous Integration
- bump to checkout v4 - (9914712) - Mattia Righetti
#### Miscellaneous Chores
- **(brew)** add brew formula - (8d03222) - Mattia Righetti
- **(ci)** bump rust to 1.74 - (7fb5ab1) - Mattia Righetti
- **(deps)** bump clap to major 4.x - (ad08c85) - Mattia Righetti
- **(deps)** update crates - (9b3ef45) - Mattia Righetti
- update man page with cog - (00d55f8) - Mattia Righetti
- update readme - (5ee1850) - Mattia Righetti
- update readme - (31051c7) - Mattia Righetti
#### Refactoring
- more examples in man page - (db23517) - Mattia Righetti

- - -

## 0.3.2 - 2023-11-18
#### Bug Fixes
- use token-tree for err macro - (314e797) - Mattia Righetti
- fix delete var in editor - (9c14bf3) - Mattia Righetti
#### Features
- edit environs from editor - (e7847ff) - Mattia Righetti
#### Miscellaneous Chores
- add install instructions - (e0a6d32) - Mattia Righetti
- update readme - (80a1362) - Mattia Righetti
#### Refactoring
- misc refactoring - (2142bf8) - Mattia Righetti
- impl std_err and err macro - (0be0ba1) - Mattia Righetti
- misc refactoring - (533a86e) - Mattia Righetti
- add err macros - (16a2e2a) - Mattia Righetti
- alphabetically order sub-commands - (5f60487) - Mattia Righetti
- isolated db queries in EnvelopeDb wrapper - (495deef) - Mattia Righetti
#### Tests
- add editor file parse and test - (86a1763) - Mattia Righetti

- - -

## 0.3.1 - 2023-10-29
#### Bug Fixes
- exit with status code 1 if err occurs - (2069f1f) - Mattia Righetti
#### Documentation
- add man page - (1f06bb1) - Mattia Righetti
#### Miscellaneous Chores
- add more stuff to man page - (076a6f8) - Mattia Righetti
#### Refactoring
- default list to raw - (48495b7) - Mattia Righetti
- move subcommand description on subcommand struct - (f531a2d) - Mattia Righetti
- standardized imports for io::Result - (aa1de2e) - Mattia Righetti

- - -

## 0.3.0 - 2023-10-27
#### Features
- **(check)** impl check command - (fede849) - Mattia Righetti

- - -

## 0.2.0 - 2023-10-26
#### Continuous Integration
- add checks on PR - (cf5b217) - Mattia Righetti
- add changelog to release notes - (4fa88a2) - Mattia Righetti
#### Features
- **(drop)** impl drop command to delete environments - (675182b) - Mattia Righetti
#### Miscellaneous Chores
- update cog config - (dc0c299) - Mattia Righetti

- - -

## 0.1.9 - 2023-10-24
#### Features
- **(list)** impl truncate feature for list - (21a627d) - Mattia Righetti
#### Miscellaneous Chores
- Cargo.lock bump - (4f54d36) - Mattia Righetti

- - -

## 0.1.8 - 2023-10-19
#### Bug Fixes
- release pipeline - (09b526d) - Mattia Righetti
#### Continuous Integration
- update ci - (416b8f2) - Mattia Righetti
- matrix build pipeline - (891b58a) - Mattia Righetti
#### Features
- **(init)** init cmd - (e626405) - Mattia Righetti
#### Miscellaneous Chores
- update cog.toml - (d749e4f) - Mattia Righetti
#### Refactoring
- renamed print methods - (d92e160) - Mattia Righetti
- take Write in import - (2395ea6) - Mattia Righetti
- new import function with reader arg - (ddc41fe) - Mattia Righetti
#### Tests
- more import tests - (e89c10f) - Mattia Righetti

- - -

## 0.1.7 - 2023-10-14
#### Bug Fixes
- **(duplicate)** duplicate only latest values and not entire history - (4db7ca0) - Mattia Righetti
- print key name and not env - (4833fc7) - Mattia Righetti
#### Continuous Integration
- initial pipeline for multi release build - (edbe321) - Mattia Righetti
#### Features
- **(list)** list all envs by default if nothing is provided - (2cea5cb) - Mattia Righetti
- **(sync)** add sync cmd with tests - (62b2563) - Mattia Righetti
- read env variable value from stdin - (2ec1ab5) - Mattia Righetti
#### Miscellaneous Chores
- move duplicate op in its own file - (edb213a) - Mattia Righetti

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).