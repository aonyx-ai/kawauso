//! Tests for the deserialization of TOML documents
//!
//! The crate deserializes the contents of a configuration file into a type
//! that the caller defines. These tests take the role of the caller: they
//! define a type, and they give a TOML document to the crate.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use indoc::indoc;
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

// config[verify load.error.syntax]
#[test]
fn from_str_with_invalid_toml_reports_the_position() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = = 8080
    "#};

    let error = kawauso_config::from_str::<Configuration>(contents).unwrap_err();

    assert_eq!(
        error.to_string(),
        "failed to parse the configuration at line 2, column 8"
    );
}

// config[verify load.error]
#[test]
fn from_str_with_invalid_toml_returns_an_error() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = = 8080
    "#};

    let result = kawauso_config::from_str::<Configuration>(contents);

    assert!(result.is_err());
}

// config[verify load.error.field]
#[test]
fn from_str_with_mismatched_field_reports_the_path() {
    let contents = indoc! {r#"
        [server]
        port = "8080"
    "#};

    let error = kawauso_config::from_str::<NestedConfiguration>(contents).unwrap_err();

    assert_eq!(
        error.to_string(),
        "failed to deserialize the configuration at `server.port`"
    );
}

// config[verify load.deserialize]
#[test]
fn from_str_with_valid_toml_returns_configuration() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = 8080
    "#};

    let configuration: Configuration = kawauso_config::from_str(contents).unwrap();

    assert_eq!(
        configuration,
        Configuration {
            name: "kawauso".to_string(),
            port: 8080,
        }
    );
}
