# kawauso-project

_Projects for Kawauso applications_

This crate finds the project that a Kawauso application runs in. A user can
then start the application in any directory of the project, and the
application still finds the files that it needs.

[`Search`] describes how the crate finds a project. It starts the search in a
given directory, or in the working directory of the process, and walks up the
directory tree until it finds a marker that identifies the project. A marker
is an entry at a relative path, such as `.git`, `src/main.rs`, or
`.config/example.toml`.

A search needs at least one marker, because a walk without one tests nothing.
The type of a search records whether it has a marker, so a search without one
does not compile.

A search that finds no project returns an error, so an application that
creates files never writes them outside a project. An application that also
runs outside a project can ask for the start directory instead.

[`Project::builder`] describes the project, and `load` then finds it. An
application opts into a configuration when it declares where the file is.
`application` puts the file at `.config/<application>.toml`, which is where
our projects keep it, and `configuration_file` names another location for an
application whose host dictates one. An application that declares neither
gets a project without a configuration.

`configuration` reports `None` for such a project, and also when no file
exists at the location that the application declared.

## Usage

```rust
use kawauso_project::Project;
use kawauso_project::Search;

use serde::Deserialize;

#[derive(Deserialize)]
struct Configuration {
    port: u16,
}

let search = Search::working_directory().marker(".git");
let project: Project<Configuration> = Project::builder()
    .application("example")
    .load(&search)?;

let port = project.configuration().map(|configuration| configuration.port);
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

[`Project::builder`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html#method.builder
[`Search`]: https://docs.rs/kawauso-project/latest/kawauso_project/search/struct.Search.html
