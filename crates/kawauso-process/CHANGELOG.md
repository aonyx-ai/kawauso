<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [0.1.0] - 2026-08-31

### Added

- Measure the grace of a stop against the command, not its streams (#132)
- Wait for the end of a command with `wait_for_end` and keep the handle (#131)
- Stop a command with `stop`, which asks before it kills (#126)
- Read a line in a `select`, because a dropped read loses no output (#123)
- Report the identifier that the operating system gave the command (#122)
- Stream the output of a command as tagged lines while it runs (#116)
- Require a successful exit status with `require_success` (#110)
- Run a command in one call and report status, output, and duration (#110)
- Name a program, its arguments, and its directory in an `Invocation` (#107)

[0.1.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-process@0.1.0
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
