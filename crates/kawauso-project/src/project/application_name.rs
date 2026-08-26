//! The name of an application

use typed_fields::name;

name!(
    /// The name of an application
    ///
    /// The name identifies the application whose project this is, and the
    /// crate derives the conventional location of the configuration file from
    /// it: `.config/<name>.toml` inside the project.
    ///
    /// Use the name that a user types to start the application, so that the
    /// user finds the file under a name that they recognize.
    ApplicationName
);
