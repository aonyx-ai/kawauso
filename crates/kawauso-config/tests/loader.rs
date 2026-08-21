//! Tests for the loading of configuration files
//!
//! The crate loads a configuration from a source and deserializes it into a
//! type that the caller defines. These tests take the role of the caller:
//! they define a type, and they give the loader contents or a file.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use indoc::indoc;
use kawauso_config::Loader;
use kawauso_config::error::LoadConfigurationError;
use serde::Deserialize;

/// The configuration of an imaginary application
///
/// A caller of the crate defines a type like this one for its own
/// configuration file. The fields cover the two kinds of value that a
/// configuration file uses most: text and numbers.
#[derive(Eq, PartialEq, Debug, Deserialize)]
struct Configuration {
    name: String,
    port: u16,
}

/// The configuration of an imaginary application that groups its settings
///
/// A configuration file often collects related settings in a table. A caller
/// that defines a type with a table like this one gets a path with a dot in
/// it when an entry of the table does not match.
#[derive(Eq, PartialEq, Debug, Deserialize)]
struct NestedConfiguration {
    server: Server,
}

/// The settings that the `server` table of a configuration file holds
#[derive(Eq, PartialEq, Debug, Deserialize)]
struct Server {
    port: u16,
}

#[test]
fn load_with_invalid_file_carries_the_cause() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("configuration.toml");
    let contents = indoc! {r#"
        name = "kawauso"
        port = "8080"
    "#};
    std::fs::write(&path, contents).unwrap();

    let error = Loader::path(&path).load::<Configuration>().unwrap_err();

    let LoadConfigurationError::InvalidFile { source, .. } = error else {
        panic!("expected the InvalidFile variant, got {error:?}");
    };
    assert_eq!(
        source.to_string(),
        "failed to deserialize the configuration at `port`"
    );
}

#[test]
fn load_with_invalid_file_reports_the_path() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("configuration.toml");
    let contents = indoc! {r#"
        name = "kawauso"
        port = "8080"
    "#};
    std::fs::write(&path, contents).unwrap();

    let error = Loader::path(&path).load::<Configuration>().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "failed to deserialize the configuration file at `{}`",
            path.display()
        )
    );
}

// config[verify load.error.syntax]
#[test]
fn load_with_invalid_toml_reports_the_position() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = = 8080
    "#};

    let error = Loader::contents(contents)
        .load::<Configuration>()
        .unwrap_err();

    let LoadConfigurationError::InvalidContents { source, .. } = error else {
        panic!("expected the InvalidContents variant, got {error:?}");
    };
    assert_eq!(
        source.to_string(),
        "failed to parse the configuration at line 2, column 8"
    );
}

// config[verify load.error]
#[test]
fn load_with_invalid_toml_returns_an_error() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = = 8080
    "#};

    let result = Loader::contents(contents).load::<Configuration>();

    assert!(result.is_err());
}

// config[verify load.error.field]
#[test]
fn load_with_mismatched_field_reports_the_path() {
    let contents = indoc! {r#"
        [server]
        port = "8080"
    "#};

    let error = Loader::contents(contents)
        .load::<NestedConfiguration>()
        .unwrap_err();

    let LoadConfigurationError::InvalidContents { source, .. } = error else {
        panic!("expected the InvalidContents variant, got {error:?}");
    };
    assert_eq!(
        source.to_string(),
        "failed to deserialize the configuration at `server.port`"
    );
}

// config[verify load.file.error.missing]
#[test]
fn load_with_missing_file_reports_the_path() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.toml");

    let error = Loader::path(&path).load::<Configuration>().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!("no configuration file exists at `{}`", path.display())
    );
}

// config[verify load.file.error]
#[test]
fn load_with_missing_file_returns_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.toml");

    let result = Loader::path(&path).load::<Configuration>();

    assert!(result.is_err());
}

// config[verify load.file.error.unreadable]
#[test]
fn load_with_unreadable_file_reports_the_path() {
    // A fixture that removes the read permission silently passes when the
    // tests run as root, such as in a container. Invalid UTF-8 fails the
    // read on every platform and for every user.
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("configuration.toml");
    std::fs::write(&path, b"name = \xff").unwrap();

    let error = Loader::path(&path).load::<Configuration>().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "failed to read the configuration file at `{}`",
            path.display()
        )
    );
}

// config[verify load.file]
#[test]
fn load_with_valid_file_returns_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("configuration.toml");
    let contents = indoc! {r#"
        name = "kawauso"
        port = 8080
    "#};
    std::fs::write(&path, contents).unwrap();

    let configuration: Configuration = Loader::path(&path).load().unwrap();

    assert_eq!(
        configuration,
        Configuration {
            name: "kawauso".to_string(),
            port: 8080,
        }
    );
}

// config[verify load.deserialize]
#[test]
fn load_with_valid_toml_returns_configuration() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = 8080
    "#};

    let configuration: Configuration = Loader::contents(contents).load().unwrap();

    assert_eq!(
        configuration,
        Configuration {
            name: "kawauso".to_string(),
            port: 8080,
        }
    );
}
