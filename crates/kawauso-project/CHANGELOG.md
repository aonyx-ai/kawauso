<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [Unreleased]

### Added

- Find the directory of a project by walking up from a given directory until
  one of the markers of the application matches
- Start the search at the working directory of the process, for an application
  that takes no path from its user
- Report the start directory as the project when no marker matches, for an
  application that also runs outside a project
- Describe a project with a builder, then find it and read its configuration
  file in one step with `Project::load`
- Read the configuration file of a project from `.config/<application>.toml`,
  or from a location that the application names, and deserialize it into a
  type that the application defines

[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
