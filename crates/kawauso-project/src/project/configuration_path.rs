//! The path at which a project keeps its configuration file

use typed_fields::path;

path!(
    /// The path at which a project keeps its configuration file
    ///
    /// The path is absolute: the directory of the project with the location
    /// of the configuration file joined onto it. A project reports this path
    /// whether or not a file exists at it, so that an application can tell
    /// its user where to put the file.
    ConfigurationPath
);
