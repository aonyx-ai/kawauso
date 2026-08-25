//! The path of the configuration file of a project

use typed_fields::path;

path!(
    /// The path of the configuration file of a project
    ///
    /// The path is the directory of the project joined with the relative
    /// path of the configuration file, which is `.config/<name>.toml` by
    /// convention. It is derived, not observed: the project derives it
    /// whether the file exists or not, so that a tool can name the place at
    /// which the file has to go.
    ConfigurationFile
);
