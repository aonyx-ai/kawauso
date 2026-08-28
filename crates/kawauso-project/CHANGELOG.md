<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [0.2.0] - 2026-08-28

### Added

- Create the configuration file of a new project with `load_or_create` (#91)
- Select the configuration directory with `with_configuration_directory` (#90)

## [0.1.0] - 2026-08-27

### Added

- Report the condition that ended a failed search in the error message (#81)
- Report the canonical path of the project, with symbolic links resolved (#80)
- Declare with `without_configuration` that an application reads no
  configuration file (#77)
- Describe a project with a builder and read its configuration in one step
  with `Project::load` (#73)
- Read the configuration file from `.config/<application>.toml` and
  deserialize it into a type that the application defines (#73)
- Fall back to the start directory when no marker matches (#72)
- Start the search at the working directory of the process (#71)
- Find the project by walking up from a directory until a marker matches
  (#69)

[0.1.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-project@0.1.0
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
