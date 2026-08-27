//! A place in which the search reads a configuration file in a directory

use std::path::Path;
use std::path::PathBuf;

use super::subdirectory::Subdirectory;
use crate::loader::ApplicationName;

/// A place in which the search reads a configuration file in a directory
///
/// The type is private to the crate. Its variants name the two kinds that an
/// application can add to the search. A location resolves the paths that it
/// reads in a directory of the walk. The vocabulary of locations stays private
/// to the crate. A third kind of location makes the type public, when the
/// shape that it needs shows itself.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum Location {
    /// A subdirectory that the developer named
    ///
    /// The location reads the file with the name of the application in the
    /// subdirectory.
    Subdirectory(Subdirectory),

    /// The dot-config convention
    ///
    /// The location reads the file with the name of the application in
    /// `.config`, and the file `config.toml` in `.config/<application>`.
    DotConfig,
}

impl Location {
    /// Returns the configuration files that this location reads in a directory
    ///
    /// A subdirectory contributes one path, and the dot-config convention
    /// contributes two. The paths never leave the directory: `.config` is a
    /// relative name that stays inside it, and so is a subdirectory that the
    /// search accepted.
    pub(crate) fn paths_in(
        &self,
        directory: &Path,
        application: &ApplicationName,
        file_name: &str,
    ) -> Vec<PathBuf> {
        match self {
            Location::Subdirectory(subdirectory) => {
                vec![directory.join(subdirectory.get()).join(file_name)]
            }
            Location::DotConfig => {
                let root = directory.join(".config");

                vec![
                    root.join(file_name),
                    root.join(application.get()).join("config.toml"),
                ]
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

    /// The name of the application whose configuration the tests search for
    const APP: &str = "example";

    /// The name that the file of the application has in most locations
    const FILE: &str = "example.toml";

    // The dot-config convention reads the file before the directory.
    // config[verify discover.ancestors.dot.files]
    #[test]
    fn dot_config_paths_in_directories_names_the_file_first() {
        let location = Location::DotConfig;
        let application = ApplicationName::new(APP);

        let paths = location.paths_in(Path::new("/project"), &application, FILE);

        assert_eq!(
            paths,
            vec![
                PathBuf::from("/project/.config/example.toml"),
                PathBuf::from("/project/.config/example/config.toml"),
            ]
        );
    }

    // A subdirectory contributes only its own file.
    #[test]
    fn subdirectory_paths_in_directories_names_its_file() {
        let location = Location::Subdirectory(Subdirectory::from(".github"));
        let application = ApplicationName::new(APP);

        let paths = location.paths_in(Path::new("/project"), &application, FILE);

        assert_eq!(paths, vec![PathBuf::from("/project/.github/example.toml")]);
    }
}
