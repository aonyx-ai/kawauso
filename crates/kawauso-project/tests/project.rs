//! Tests for the project of an application
//!
//! The crate finds the project of an application and loads its
//! configuration. These tests take the role of the application: they build a
//! search, discover the project in a directory that they prepared, and ask
//! the project for its configuration.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::Path;

use kawauso_project::Project;
use kawauso_project::ProjectSearch;
use kawauso_project::error::LoadProjectConfigurationError;
use serde::Deserialize;

/// The name of the application whose project the tests search for
const APPLICATION: &str = "kawauso";

/// The configuration of an imaginary application
///
/// A caller of the crate defines a type like this one for its own
/// configuration file. The default stands in for a project without a file.
#[derive(Eq, PartialEq, Debug, Default, Deserialize)]
struct Configuration {
    #[serde(default)]
    port: u16,
}

/// Prepares a project directory that holds the given entries
///
/// Each entry is a relative path, and the directories on the way are created
/// as well. An entry with contents is a file with those contents.
fn project_with(entries: &[(&str, &str)]) -> tempfile::TempDir {
    // The lints that ban `unwrap` and `expect` make an exception for a test,
    // and a helper of a test is not one, so each failure gets its own report.
    let Ok(directory) = tempfile::tempdir() else {
        panic!("the test needs a temporary directory");
    };

    for (entry, contents) in entries {
        let path = directory.path().join(entry);
        let Some(parent) = path.parent() else {
            panic!("the entry `{entry}` names no directory to go in");
        };
        assert!(
            std::fs::create_dir_all(parent).is_ok(),
            "the test needs the directory `{}`",
            parent.display()
        );
        assert!(
            std::fs::write(&path, contents).is_ok(),
            "the test needs the file `{}`",
            path.display()
        );
    }

    directory
}

// project[verify configuration.location]
#[test]
fn configuration_file_returns_the_conventional_location() {
    let directory = project_with(&[(".git", "")]);
    let search = ProjectSearch::new(APPLICATION)
        .marker(".git")
        .start(directory.path());

    let project = Project::discover(search).unwrap();

    assert_eq!(
        project.configuration_file().get(),
        directory.path().join(".config").join("kawauso.toml")
    );
}

// project[verify configuration.location.custom]
#[test]
fn configuration_file_with_a_custom_location_returns_that_location() {
    let directory = project_with(&[(".github/kawauso.toml", "port = 8080")]);
    let search = ProjectSearch::new(APPLICATION)
        .configuration_file(".github/kawauso.toml")
        .start(directory.path());

    let project = Project::discover(search).unwrap();

    assert_eq!(
        project.configuration_file().get(),
        directory.path().join(".github").join("kawauso.toml")
    );
}

// project[verify configuration.default]
#[test]
fn configuration_or_default_with_a_file_returns_the_configuration() {
    let directory = project_with(&[(".config/kawauso.toml", "port = 8080")]);
    let search = ProjectSearch::new(APPLICATION).start(directory.path());
    let project = Project::discover(search).unwrap();

    let configuration: Configuration = project.configuration_or_default().unwrap();

    assert_eq!(configuration, Configuration { port: 8080 });
}

// project[verify configuration.default.error]
#[test]
fn configuration_or_default_with_an_invalid_file_returns_an_error() {
    let directory = project_with(&[(".config/kawauso.toml", "port = \"otter\"")]);
    let search = ProjectSearch::new(APPLICATION).start(directory.path());
    let project = Project::discover(search).unwrap();

    let result = project.configuration_or_default::<Configuration>();

    assert!(matches!(
        result,
        Err(LoadProjectConfigurationError::UnloadableConfiguration { .. })
    ));
}

// project[verify configuration.default]
#[test]
fn configuration_or_default_without_a_file_returns_the_default() {
    let directory = project_with(&[(".git", "")]);
    let search = ProjectSearch::new(APPLICATION)
        .marker(".git")
        .start(directory.path());
    let project = Project::discover(search).unwrap();

    let configuration: Configuration = project.configuration_or_default().unwrap();

    assert_eq!(configuration, Configuration::default());
}

// project[verify configuration.load]
#[test]
fn configuration_with_a_file_returns_the_configuration() {
    let directory = project_with(&[(".config/kawauso.toml", "port = 8080")]);
    let search = ProjectSearch::new(APPLICATION).start(directory.path());
    let project = Project::discover(search).unwrap();

    let configuration: Configuration = project.configuration().unwrap();

    assert_eq!(configuration, Configuration { port: 8080 });
}

// The message of the cause names the file, so that the user knows which file
// to correct, while the error of the project names the project.
// project[verify configuration.error]
#[test]
fn configuration_with_an_invalid_file_names_the_project() {
    let directory = project_with(&[(".config/kawauso.toml", "port = \"otter\"")]);
    let search = ProjectSearch::new(APPLICATION).start(directory.path());
    let project = Project::discover(search).unwrap();

    let error = project.configuration::<Configuration>().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "failed to load the configuration of the project at `{}`",
            directory.path().display()
        )
    );
}

// project[verify configuration.error]
#[test]
fn configuration_with_an_invalid_file_returns_an_error() {
    let directory = project_with(&[(".config/kawauso.toml", "port = \"otter\"")]);
    let search = ProjectSearch::new(APPLICATION).start(directory.path());
    let project = Project::discover(search).unwrap();

    let result = project.configuration::<Configuration>();

    assert!(matches!(
        result,
        Err(LoadProjectConfigurationError::UnloadableConfiguration { .. })
    ));
}

// project[verify configuration.error.missing]
#[test]
fn configuration_without_a_file_names_the_path() {
    let directory = project_with(&[(".git", "")]);
    let search = ProjectSearch::new(APPLICATION)
        .marker(".git")
        .start(directory.path());
    let project = Project::discover(search).unwrap();

    let error = project.configuration::<Configuration>().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "no configuration file exists at `{}`",
            directory
                .path()
                .join(".config")
                .join("kawauso.toml")
                .display()
        )
    );
}

// project[verify configuration.error.missing]
#[test]
fn configuration_without_a_file_returns_an_error() {
    let directory = project_with(&[(".git", "")]);
    let search = ProjectSearch::new(APPLICATION)
        .marker(".git")
        .start(directory.path());
    let project = Project::discover(search).unwrap();

    let result = project.configuration::<Configuration>();

    assert!(matches!(
        result,
        Err(LoadProjectConfigurationError::MissingFile { .. })
    ));
}

// Cargo runs the tests of a crate in the directory of the crate, which holds
// `Cargo.toml` and `src`. A start that is relative resolves against that
// directory, so the walk from `src` finds the manifest one level up.
// project[verify discover.start.absolute]
#[test]
fn discover_with_a_relative_start_resolves_it_against_the_working_directory() {
    let working_directory = std::env::current_dir().unwrap();
    let search = ProjectSearch::new(APPLICATION)
        .marker("Cargo.toml")
        .start(Path::new("src"));

    let project = Project::discover(search).unwrap();

    assert_eq!(project.root().get(), working_directory);
}

// project[verify discover.start.caller]
#[test]
fn discover_with_a_start_starts_there() {
    let directory = project_with(&[(".git", "")]);
    let search = ProjectSearch::new(APPLICATION)
        .marker(".git")
        .start(directory.path());

    let project = Project::discover(search).unwrap();

    assert_eq!(project.root().get(), directory.path());
}
