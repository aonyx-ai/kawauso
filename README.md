# 🦦 Kawauso

Kawauso, named after the Japanese river otter, is a toolkit for building
applications in Rust. It provides a set of crates that can be used together or
independently to implement features that most applications share.

## Development

[Mise] provisions every tool that this repository needs, at the versions that
`mise.toml` pins:

```console
mise install
```

The first installation builds [Tracey] from source, which takes several
minutes. Later installations reuse the binary.

Activate mise in your shell, as the [mise documentation][mise-activate]
describes. Activation puts the pinned tools on your `PATH`, and with them the
`bin` directory of the repository, which holds a stub for every mise task.

[Rakko] provides the maintenance tasks. The package in `tools/rakko` is the
harness: it mounts the actions that maintain this repository, and it turns
each of them into a command. A run without a command lists them:

```console
mise run rakko
```

Where mise supplies its environment, `rakko` is the same command from every
directory of the repository.

`pre-commit install` installs the Git hook, which runs
`rakko pre-commit --fix` before every commit.

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

[mise]: https://mise.jdx.dev
[mise-activate]: https://mise.jdx.dev/getting-started.html#activate-mise
[rakko]: https://github.com/aonyx-ai/rakko
[tracey]: https://tracey.bearcove.eu/
