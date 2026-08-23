# kawauso-config

Configuration files for Kawauso applications

This crate loads the configuration files of Kawauso applications. Every
application finds, loads, and deserializes its configuration file in the same
way, and reports failures with the same clear errors.

[`Loader`] is the entry point of the crate. A constructor selects the source
of the configuration, and [`load`] deserializes the configuration into a type
that the caller defines.

The source can be a search. An application whose configuration belongs to a
project uses [`ancestors`], which walks from the working directory up to the
root of the file system. An application whose configuration belongs to a user
uses [`user`], which reads the directory that the platform defines for the
configuration of a user.

## Usage

```rust
use serde::Deserialize;

use kawauso_config::Loader;

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

[`ancestors`]: https://docs.rs/kawauso-config/latest/kawauso_config/loader/struct.Loader.html#method.ancestors
[`load`]: https://docs.rs/kawauso-config/latest/kawauso_config/loader/struct.Loader.html#method.load
[`Loader`]: https://docs.rs/kawauso-config/latest/kawauso_config/loader/struct.Loader.html
[`user`]: https://docs.rs/kawauso-config/latest/kawauso_config/loader/struct.Loader.html#method.user
