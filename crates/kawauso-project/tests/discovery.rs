//! Tests for the discovery of a project through the public API
//!
//! The unit tests of the crate reach the walk and the resolution of the start
//! directly, so that each of them can name a start and a working directory of
//! its own. A test here goes through [`Project::discover`], which puts the
//! two together, and checks that the walk begins where the search says.

// An assertion in a test panics by design. A `# Panics` section on every test
// would repeat that and give the reader no information.
#![allow(clippy::missing_panics_doc)]

use kawauso_project::Project;
use kawauso_project::Search;

// project[verify discover.start.caller]
#[test]
fn discover_with_a_start_in_one_of_two_projects_returns_that_project() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    std::fs::create_dir_all(first.join(".git")).unwrap();
    std::fs::create_dir_all(second.join(".git")).unwrap();
    std::fs::create_dir_all(second.join("src")).unwrap();

    let search = Search::start(second.join("src")).marker(".git");
    let project = Project::discover(&search).unwrap();

    assert_eq!(project.root().get(), second);
}
