//! A place in which the search reads a configuration file in a directory

use std::path::Path;
use std::path::PathBuf;

use super::subdirectory::Subdirectory;

/// A place in which the search reads a configuration file in a directory
///
/// The type is private to the crate. Its variants name the kinds that an
/// application can add to the search. A location resolves the paths that it
/// reads in a directory of the walk, so a kind that reads more than one file
/// needs no change anywhere else. The vocabulary of locations stays private
/// to the crate. A second kind of location makes the type public, when the
/// shape that it needs shows itself.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum Location {
    /// A subdirectory that the developer named
    ///
    /// The location reads the file with the name of the application in the
    /// subdirectory.
    Subdirectory(Subdirectory),
}

impl Location {
    /// Returns the configuration files that this location reads in a directory
    ///
    /// A subdirectory contributes one path. The paths never leave the
    /// directory, because a subdirectory that the search accepted stays
    /// inside it.
    pub(crate) fn paths_in(&self, directory: &Path, file_name: &str) -> Vec<PathBuf> {
        match self {
            Location::Subdirectory(subdirectory) => {
                vec![directory.join(subdirectory.get()).join(file_name)]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    /// The name that the file of the application has in a location
    const FILE: &str = "example.toml";

    // A subdirectory contributes only its own file.
    #[test]
    fn subdirectory_paths_in_directories_names_its_file() {
        let location = Location::Subdirectory(Subdirectory::from(".github"));

        let paths = location.paths_in(Path::new("/project"), FILE);

        assert_eq!(paths, vec![PathBuf::from("/project/.github/example.toml")]);
    }
}
