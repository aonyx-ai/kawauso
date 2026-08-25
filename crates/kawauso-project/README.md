# kawauso-project

_Projects for Kawauso applications_

This crate finds the project that a Kawauso application runs in. Every
application finds its project in the same way, and it anchors the relative
paths of its resources at the same directory.

[`Search`] describes how the crate finds a project. It names the directory at
which the walk starts, and the markers that identify the project. A marker is
an entry at a relative path, such as `.git`, `src/main.rs`, or
`.config/example.toml`. The crate tests only whether the entry exists, and it
never reads the entry.

[`discover`] walks from the start up to the root of the file system. It tests
every marker in one directory before it tests any marker in the directory
above. The first directory that holds a marker is the project. The crate
reports this directory together with the marker that matched, so no
application derives the directory from the path of a file.

A search needs at least one marker, because a walk without one tests nothing.
The type of a search records whether it has a marker, so a search without one
does not compile.

## Usage

```rust
use kawauso_project::Project;
use kawauso_project::Search;

let search = Search::start("src/main.rs").marker(".git");
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

[`discover`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html#method.discover
[`Search`]: https://docs.rs/kawauso-project/latest/kawauso_project/search/struct.Search.html
