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

## Usage

```rust
use kawauso_project::Project;
use kawauso_project::Search;

let search = Search::working_directory().marker(".git");
let project = Project::discover(&search)?;

let manifest = project.root().get().join("Cargo.toml");
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

[`Search`]: https://docs.rs/kawauso-project/latest/kawauso_project/search/struct.Search.html
