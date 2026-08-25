# kawauso-project

Projects for Kawauso applications

This crate finds the project that a Kawauso application runs in. Every
application finds its project in the same way, anchors its relative paths at
the same directory, and reads its configuration file from the same
conventional location.

[`Project`] is the entry point of the crate. [`discover`] walks up from a
directory until it finds one that holds a marker of the application, such as
its configuration file or `.git`, and it returns that directory as the
project. A [`ProjectSearch`] names the markers, the directory at which the
walk starts, and what the search does when no marker matches.

A project keeps the configuration file of an application at
`.config/<application>.toml`. [`configuration`] loads that file into a type
that the application defines, and [`configuration_or_default`] returns the
default of that type when the project has no configuration file. Both use
[`kawauso-config`][config] to read and deserialize the file.

## Usage

```rust
use kawauso_project::Project;
use kawauso_project::ProjectSearch;
use serde::Deserialize;

#[derive(Default, Deserialize)]
struct Configuration {
    #[serde(default)]
    ignore: Vec<String>,
}

// Finds the first directory at or above the working directory that holds
// `.config/example.toml` or `.git`
let search = ProjectSearch::new("example").marker(".git");
let project = Project::discover(search)?;

// Loads `.config/example.toml`, or falls back to the default of the type
let configuration: Configuration = project.configuration_or_default()?;

println!("{} ignores {} patterns", project.root(), configuration.ignore.len());
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

[`configuration`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html#method.configuration
[`configuration_or_default`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html#method.configuration_or_default
[`discover`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html#method.discover
[`Project`]: https://docs.rs/kawauso-project/latest/kawauso_project/project/struct.Project.html
[`ProjectSearch`]: https://docs.rs/kawauso-project/latest/kawauso_project/search/struct.ProjectSearch.html
[config]: https://docs.rs/kawauso-config
