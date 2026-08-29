# kawauso-process

_External programs for Kawauso applications_

This crate runs the external programs of a Kawauso application. Every
application starts a program in the same way. The caller names the program and
its arguments. The crate collects the output of the program, and it reports how
the program ended.

## Asynchronous Runtime

The crate runs a command asynchronously, and it needs a [Tokio] runtime. A run
starts the program, reads both of its streams, and waits for the end of the
command, and the runtime drives that work. A run that no Tokio runtime drives
panics.

An application that already has a runtime, because it starts one in its own
entry point, calls the crate from a task of that runtime. An application
without one starts a runtime for the runs that it makes.

## Windows

A program whose name ends in `.cmd` or `.bat` starts through `cmd.exe`. A tool
that npm installs arrives in this form. Rust escapes the arguments for that
interpreter, and it refuses an argument that it cannot pass safely. An
argument therefore reaches the program as the caller wrote it, or the run
fails and says so.

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

[tokio]: https://tokio.rs
