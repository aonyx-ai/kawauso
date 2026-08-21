//! The name of an application that has a configuration file

use typed_fields::name;

name!(
    /// The name of an application that has a configuration file
    ///
    /// A search derives the name of the file from the name of the
    /// application, so that the developer supplies one word and gets the
    /// same file name on every platform. A search near the working
    /// directory looks for `<name>.toml`, and a search in the directory of
    /// the user looks for `config.toml` in a directory `<name>`.
    ///
    /// The name is stored as the developer gave it. It becomes part of a
    /// path, so it must be a name that the file system of the platform
    /// accepts.
    ApplicationName
);
