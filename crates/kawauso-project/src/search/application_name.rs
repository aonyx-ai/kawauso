//! The name of an application that has a project

use typed_fields::name;

name!(
    /// The name of an application that has a project
    ///
    /// The search derives the configuration file of the application from
    /// its name: a project keeps the file `<name>.toml` in its subdirectory
    /// `.config`. The developer supplies one word, and every project of the
    /// application keeps the file at the same place.
    ///
    /// The name is stored as the developer gave it. It becomes part of a
    /// path, so it must be a name that the file system of the platform
    /// accepts.
    ApplicationName
);
