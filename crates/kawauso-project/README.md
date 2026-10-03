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

[`Project::builder`] describes the project, and `load` then finds it and
reads its configuration file. Every project belongs to an application, and the
name of the application decides where that file is:
`.config/<application>.toml`, which is where our projects keep it. An
application whose host dictates another location names it with
`configuration_file`.

The name of the application becomes a component of a path. It therefore holds
from 1 to 250 ASCII letters, ASCII digits, and hyphens, and it is not the name
of a device on Windows, such as `CON` or `COM1`. The application creates the
name before it describes the project, and a malformed name fails there, not in
the load.

An application that owns more than a configuration file keeps its directory in
`.config` and selects it with `with_configuration_directory`. The project then
reads `config.toml` in `.config/<application>`, which the application names
once.

A project without the file is a normal state and not a failure.
`configuration` reports `None` for it, and `configuration_path` tells the user
where to put the file.

An application whose work needs the file creates it with `load_or_create`. The
caller supplies the value, because an application that keeps an identifier in
the file makes that value for one project. No constant describes such a value.
Creation is the only write: a file that a user wrote holds their comments and
the order of their fields. The crate therefore reports a file that it cannot
read, and never repairs one.

Some applications have no configuration file at all, and want only the
directory of their project. Such an application declares this with
`without_configuration`, and the project then reads no file. A file at the
conventional location belongs to something else, and the project leaves it
alone. `configuration_path` still reports that location, so an application
that writes the file later knows where the file goes.

An application sometimes writes files that must not be in the repository,
such as a log for each run. `data_directory` gives a loaded project a
directory for them in the local data directory of the user:
`<data>/<application>/projects/<identifier>`. The application supplies the
identifier, because only the application knows what makes two checkouts the
same project. The identifier obeys the rules of the name of the application.
The call creates the directory, and the load does not.

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
    .application("example".parse()?)
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
