# kawauso

_A toolkit for building Rust applications_

Kawauso is a framework, and each part of it is also a crate that an
application can use on its own. This crate is the framework as a whole: it
re-exports the other crates as modules, so that an application needs one
dependency and one version requirement to reach all of them.

A module carries the name of its crate without the prefix `kawauso-`, and the
module is that crate rather than a copy of it. `kawauso::config::Loader` and
`kawauso_config::Loader` are one type, so an application that depends on this
crate and a library that depends on the single crate can pass values to each
other.

This crate has no features. It brings the whole framework, and an application
that wants a part of it depends on the crates that hold that part.

## Modules

| Module    | Crate                        | Description                                 |
| --------- | ---------------------------- | ------------------------------------------- |
| `config`  | [`kawauso-config`][config]   | Configuration files for the application     |
| `process` | [`kawauso-process`][process] | External programs that the application runs |
| `project` | [`kawauso-project`][project] | The project that the application runs in    |

## Usage

```rust
use serde::Deserialize;

use kawauso::config::Loader;

#[derive(Deserialize)]
struct Configuration {
    port: u16,
}

let configuration: Configuration = Loader::contents("port = 8080").load()?;

assert_eq!(configuration.port, 8080);
```

## License

Copyright (c) 2026 Aonyx B.V.

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT)
  or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[config]: https://docs.rs/kawauso-config
[process]: https://docs.rs/kawauso-process
[project]: https://docs.rs/kawauso-project
