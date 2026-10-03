<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [0.4.0] - 2026-10-03

### Changed

- Accept only portable names of applications, with 1 to 250 ASCII letters,
  digits, and hyphens and no device names of Windows (#163, #164)

### Removed

- Remove `ApplicationName::new` and the conversions from `&str` and `String`
  into `ApplicationName` and `AncestorsSearch` (#163)

## [0.3.0] - 2026-08-28

### Added

- Support the dot-config convention in the ancestors search (#88)

### Removed

- Remove the `subdirectories` getter on `AncestorsSearch` (#87)

## [0.2.0] - 2026-08-24

### Changed

- Optionally also search specific subdirectories in ancestor search (#59)

## [0.1.0] - 2026-08-24

### Added

- Find a configuration file in TOML format using different strategies, and load
  and deserialize it into a Rust struct

[0.1.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-config@0.1.0
[0.2.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-config@0.2.0
[0.3.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-config@0.3.0
[0.4.0]: https://github.com/aonyx-ai/kawauso/releases/tag/kawauso-config@0.4.0
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
