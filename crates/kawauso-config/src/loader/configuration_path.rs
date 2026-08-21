//! The path of a configuration file

use typed_fields::path;

path!(
    /// The path of a configuration file
    ///
    /// The path is stored as the caller gave it: the crate does not
    /// canonicalize it, and it does not check that a file exists. The
    /// `Display` implementation shows the path the way the operating system
    /// renders it, so an error message can name the file.
    ConfigurationPath
);
