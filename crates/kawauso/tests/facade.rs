//! Verification of the facade
//!
//! The crate has no code of its own. These tests check its shape. A crate of
//! the toolkit is reachable under its module. The module is that crate and not
//! a copy of it.

#![allow(
    clippy::missing_panics_doc,
    reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
)]

use std::collections::BTreeMap;

/// Shows that the config module is the `kawauso-config` crate
///
/// The annotation is the assertion: it only compiles when the module and
/// the crate name one type.
#[test]
fn config_module_is_the_kawauso_config_crate() {
    let loader: kawauso_config::Loader = kawauso::config::Loader::contents("port = 8080");

    let configuration: BTreeMap<String, u16> = loader.load().unwrap();

    assert_eq!(configuration["port"], 8080);
}

#[test]
fn config_module_provides_the_configuration_loader() {
    let loader = kawauso::config::Loader::contents("port = 8080");

    let configuration: BTreeMap<String, u16> = loader.load().unwrap();

    assert_eq!(configuration["port"], 8080);
}

/// Shows that the process module is the `kawauso-process` crate
///
/// The annotation is the assertion: it only compiles when the module and
/// the crate name one type.
#[test]
fn process_module_is_the_kawauso_process_crate() {
    let invocation: kawauso_process::Invocation =
        kawauso::process::Invocation::new("git").arg("status");

    let command = invocation.to_string();

    assert_eq!(command, "git status");
}

#[test]
fn process_module_provides_the_command_invocation() {
    let invocation = kawauso::process::Invocation::new("git").arg("status");

    let command = invocation.to_string();

    assert_eq!(command, "git status");
}

/// Shows that the project module is the `kawauso-project` crate
///
/// The annotation is the assertion: it only compiles when the module and
/// the crate name one type.
#[test]
fn project_module_is_the_kawauso_project_crate() {
    let search: kawauso_project::Search<kawauso_project::search::Marked> =
        kawauso::project::Search::start("src/main.rs").marker(".git");

    let markers = search.markers().len();

    assert_eq!(markers, 1);
}

#[test]
fn project_module_provides_the_project_search() {
    let search = kawauso::project::Search::start("src/main.rs").marker(".git");

    let markers = search.markers().len();

    assert_eq!(markers, 1);
}
