//! Tests for the configuration that a project holds
//!
//! A project reads its configuration file when it loads. A test here writes a
//! project into a temporary directory, loads it, and asks it for the
//! configuration. Every test names its own start directory, so the tests need
//! no environment of their own and run beside each other.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::path::PathBuf;

use kawauso_project::Project;
use kawauso_project::Search;
use kawauso_project::project::NoConfiguration;
use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

/// The name of the application whose project the tests load
const APPLICATION: &str = "example";

/// The contents of the configuration file that the tests write
const CONTENTS: &str = "port = 8080";

/// The marker that identifies the projects of these tests
const MARKER: &str = ".git";

/// The contents of a configuration file that its author formatted
///
/// A serializer that writes the whole document loses the comment and the
/// blank line. A test that asserts that the crate wrote nothing therefore
/// sees the difference.
const FORMATTED_CONTENTS: &str = "# The port that the application listens on\n\nport = 8080\n";

/// The configuration of an imaginary application
///
/// An application of the crate defines a type like this one for its own
/// configuration file. One field is enough here, because these tests are
/// about the file that the project reads, not about the shape of a
/// configuration.
///
/// The type serializes as well, because a project creates its configuration
/// file from a value of the type of the application.
#[derive(Debug, Deserialize, Serialize)]
struct Configuration {
    port: u16,
}

/// A configuration that no TOML document can hold
///
/// A TOML document is always a table, and a number is not one, so a
/// serializer rejects this value. The type gives a test a configuration that
/// the crate cannot write, without a file system that refuses the write.
#[derive(Debug, Deserialize, Serialize)]
struct UnserializableConfiguration(u16);

/// Returns the canonical path of a temporary directory
///
/// A temporary directory can sit below a symbolic link, which is what macOS
/// does for `/var`. A project reports canonical paths, so a test that names a
/// location in the directory canonicalizes it first.
///
/// The lints that ban `unwrap` and `expect` make an exception for a test, and
/// a helper of a test is not one. The failure therefore travels to the test,
/// which is allowed to panic on it.
///
/// # Errors
///
/// Returns an error when the directory has no canonical path.
fn canonical(directory: &TempDir) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(directory.path())
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

// A file that the type of the application rejects fails the load. The crate
// reports the problem instead of a repair that loses what the author wrote.
// project[verify configuration.create.existing]
#[test]
fn load_or_create_with_a_broken_configuration_file_keeps_the_file() {
    let directory = project(Some((".config/example.toml", "port = "))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let _ = Project::<Configuration>::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 9090 });

    assert_eq!(
        std::fs::read_to_string(directory.path().join(".config").join("example.toml")).unwrap(),
        "port = "
    );
}

// project[verify configuration.create.existing]
#[test]
fn load_or_create_with_a_broken_configuration_file_returns_an_error() {
    let directory = project(Some((".config/example.toml", "port = "))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let error = Project::<Configuration>::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 9090 })
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("failed to load the configuration")
    );
}

// project[verify configuration.create.existing]
#[test]
fn load_or_create_with_a_configuration_file_keeps_the_file() {
    let directory = project(Some((".config/example.toml", FORMATTED_CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let _: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 9090 })
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(directory.path().join(".config").join("example.toml")).unwrap(),
        FORMATTED_CONTENTS
    );
}

// project[verify configuration.create.existing]
#[test]
fn load_or_create_with_a_configuration_file_reads_it() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 9090 })
        .unwrap();

    assert_eq!(project.configuration().unwrap().port, 8080);
}

// project[verify configuration.create.error]
#[test]
fn load_or_create_with_an_unserializable_configuration_returns_an_error() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let error = Project::<UnserializableConfiguration>::builder()
        .application(APPLICATION)
        .load_or_create(&search, || UnserializableConfiguration(8080))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("failed to create the configuration")
    );
}

// project[verify configuration.create]
#[test]
fn load_or_create_without_a_configuration_file_creates_it() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let _: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 8080 })
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(directory.path().join(".config").join("example.toml")).unwrap(),
        "port = 8080\n"
    );
}

// The conventional directory adds a second level below the project, and a
// project that never held a configuration file has neither of them.
// project[verify configuration.create.directories]
#[test]
fn load_or_create_without_a_configuration_file_creates_the_directories() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let _: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .with_configuration_directory()
        .load_or_create(&search, || Configuration { port: 8080 })
        .unwrap();

    assert!(
        directory
            .path()
            .join(".config")
            .join(APPLICATION)
            .join("config.toml")
            .is_file()
    );
}

// project[verify configuration.create.result]
#[test]
fn load_or_create_without_a_configuration_file_reports_the_value() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load_or_create(&search, || Configuration { port: 9090 })
        .unwrap();

    assert_eq!(project.configuration().unwrap().port, 9090);
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

// The project reads one location, so a file at the other layout belongs to
// something else and stays untouched.
// project[verify configuration.location.directory]
#[test]
fn load_with_a_configuration_directory_ignores_the_file_layout() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .with_configuration_directory()
        .load(&search)
        .unwrap();

    assert!(project.configuration().is_none());
}

// project[verify configuration.location.directory]
#[test]
fn load_with_a_configuration_directory_reads_its_file() {
    let directory = project(Some((".config/example/config.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .with_configuration_directory()
        .load(&search)
        .unwrap();

    assert_eq!(project.configuration().unwrap().port, 8080);
}

// project[verify configuration.location.directory]
#[test]
fn load_with_a_configuration_directory_reports_the_directory() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .with_configuration_directory()
        .load(&search)
        .unwrap();

    assert_eq!(
        project.configuration_path().get(),
        canonical(&directory)
            .unwrap()
            .join(".config")
            .join("example")
            .join("config.toml")
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
        canonical(&directory)
            .unwrap()
            .join(".github")
            .join("example.toml")
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
        canonical(&directory)
            .unwrap()
            .join(".config")
            .join("example.toml")
    );
}

// A read that writes surprises its caller, so a project without a
// configuration file stays without one until an application asks for the
// creation.
// project[verify configuration.create.load]
#[test]
fn load_without_a_configuration_file_creates_no_file() {
    let directory = project(None).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let _: Project<Configuration> = Project::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap();

    assert!(!directory.path().join(".config").exists());
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

// A file at the conventional location belongs to something else when the
// application declared that it has no configuration file. The project leaves
// that file alone, so contents that no type of the application describes
// cannot fail the load.
// project[verify configuration.none]
#[test]
fn load_without_a_configuration_ignores_a_file_at_the_location() {
    let directory = project(Some((".config/example.toml", "port = "))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Result<Project, _> = Project::builder()
        .application(APPLICATION)
        .without_configuration()
        .load(&search);

    assert!(project.is_ok());
}

// project[verify configuration.none]
#[test]
fn load_without_a_configuration_reports_no_configuration() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let project: Project = Project::builder()
        .application(APPLICATION)
        .without_configuration()
        .load(&search)
        .unwrap();

    assert!(project.configuration().is_none());
}

// A project that reads a file into `NoConfiguration` gets contents that no
// type of the application describes. The load fails, which tells the
// developer that the application reads a file that it never described.
#[test]
fn load_without_a_configuration_type_and_with_a_file_returns_an_error() {
    let directory = project(Some((".config/example.toml", CONTENTS))).unwrap();
    let search = Search::start(directory.path()).marker(MARKER);

    let error = Project::<NoConfiguration>::builder()
        .application(APPLICATION)
        .load(&search)
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("failed to load the configuration")
    );
}
