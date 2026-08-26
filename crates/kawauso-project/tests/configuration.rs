//! Tests for the configuration that a project holds
//!
//! A project reads its configuration file when it loads. A test here writes a
//! project into a temporary directory, loads it, and asks it for the
//! configuration. Every test names its own start directory, so the tests need
//! no environment of their own and run beside each other.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use kawauso_project::Project;
use kawauso_project::Search;
use serde::Deserialize;
use tempfile::TempDir;

/// The name of the application whose project the tests load
const APPLICATION: &str = "example";

/// The contents of the configuration file that the tests write
const CONTENTS: &str = "port = 8080";

/// The marker that identifies the projects of these tests
const MARKER: &str = ".git";

/// The configuration of an imaginary application
///
/// An application of the crate defines a type like this one for its own
/// configuration file. One field is enough here, because these tests are
/// about the file that the project reads, not about the shape of a
/// configuration.
#[derive(Debug, Deserialize)]
struct Configuration {
    port: u16,
}

/// Creates a project with a marker, and a file at the path when one is given
///
/// The lints that ban `unwrap` and `expect` make an exception for a test, and
/// a helper of a test is not one. The failures therefore travel to the test,
/// which is allowed to panic on them.
///
/// # Errors
///
/// Returns an error when the temporary directory, the marker, or the
/// configuration file cannot be created.
fn project(configuration: Option<(&str, &str)>) -> std::io::Result<TempDir> {
    let directory = tempfile::tempdir()?;
    std::fs::create_dir(directory.path().join(MARKER))?;

    if let Some((path, contents)) = configuration {
        let file = directory.path().join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, contents)?;
    }

    Ok(directory)
}

// project[verify configuration.error]
#[test]
fn load_with_a_broken_configuration_file_returns_an_error() {
    let directory = project(Some((".config/example.toml", "port = "))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let error = Project::<Configuration>::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("failed to load the configuration")
    );
}

// project[verify configuration.load]
#[test]
fn load_with_a_configuration_file_deserializes_it() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap();

    assert_eq!(project.configuration().unwrap().port, 8080);
}

// project[verify configuration.location.custom]
#[test]
fn load_with_a_custom_location_reads_the_file_at_that_location() {
    let directory = project(Some((".github/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .configuration_file(".github/example.toml")
        .load(&search)
        .unwrap();

    assert_eq!(
        project.configuration_path().get(),
        directory.path().join(".github").join("example.toml")
    );
}

// project[verify configuration.location]
#[test]
fn load_with_an_application_name_reads_the_conventional_location() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap();

    assert_eq!(
        project.configuration_path().get(),
        directory.path().join(".config").join("example.toml")
    );
}

// project[verify configuration.missing]
#[test]
fn load_without_a_configuration_file_reports_no_configuration() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap();

    assert!(project.configuration().is_none());
}
