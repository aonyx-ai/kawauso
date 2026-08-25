//! Verification of the facade
//!
//! The crate has no code of its own, so these tests check its shape: that a
//! crate of the toolkit is reachable under its module, and that the module is
//! that crate and not a copy of it.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;

// kawauso[verify facade.identity]
#[test]
fn config_module_is_the_kawauso_config_crate() {
    // The annotation is the assertion: it only compiles when the module and
    // the crate name one type.
    let loader: kawauso_config::Loader = kawauso::config::Loader::contents("port = 8080");

    let configuration: BTreeMap<String, u16> = loader.load().unwrap();

    assert_eq!(configuration["port"], 8080);
}

// kawauso[verify facade.module]
#[test]
fn config_module_provides_the_configuration_loader() {
    let loader = kawauso::config::Loader::contents("port = 8080");

    let configuration: BTreeMap<String, u16> = loader.load().unwrap();

    assert_eq!(configuration["port"], 8080);
}

// kawauso[verify facade.identity]
#[test]
fn project_module_is_the_kawauso_project_crate() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join(".git")).unwrap();
    // The annotation is the assertion: it only compiles when the module and
    // the crate name one type.
    let search: kawauso_project::ProjectSearch = kawauso::project::ProjectSearch::new("kawauso")
        .marker(".git")
        .start(directory.path());

    let project = kawauso::project::Project::discover(search).unwrap();

    assert_eq!(project.root().get(), directory.path());
}

// kawauso[verify facade.module]
#[test]
fn project_module_provides_the_project_search() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join(".git")).unwrap();
    let search = kawauso::project::ProjectSearch::new("kawauso")
        .marker(".git")
        .start(directory.path());

    let project = kawauso::project::Project::discover(search).unwrap();

    assert_eq!(project.root().get(), directory.path());
}
