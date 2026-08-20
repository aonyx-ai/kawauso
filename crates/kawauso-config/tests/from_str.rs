//! Tests for the deserialization of TOML documents
//!
//! The crate deserializes the contents of a configuration file into a type
//! that the caller defines. These tests take the role of the caller: they
//! define a type, and they give a TOML document to the crate.

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

// config[verify load.deserialize]
#[test]
fn from_str_with_valid_toml_returns_configuration() {
    let contents = indoc! {r#"
        name = "kawauso"
        port = 8080
    "#};

    let configuration: Configuration = kawauso_config::from_str(contents);

    assert_eq!(
        configuration,
        Configuration {
            name: "kawauso".to_string(),
            port: 8080,
        }
    );
}
