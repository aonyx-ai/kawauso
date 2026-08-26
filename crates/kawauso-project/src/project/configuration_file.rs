//! The location of the configuration file inside a project

use typed_fields::path;

path!(
    /// The location of the configuration file inside a project
    ///
    /// The value is a path relative to the directory of the project, such as
    /// `.config/example.toml`. The crate derives it from the name of the
    /// application unless the developer names another location, which an
    /// application whose host dictates one needs.
    ///
    /// The path is stored as the developer gave it. It joins onto the
    /// directory of the project when the project loads.
    ConfigurationFile
);
