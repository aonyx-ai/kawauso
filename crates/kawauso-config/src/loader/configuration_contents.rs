//! The contents of a configuration file

use typed_fields::name;

name!(
    /// The contents of a configuration file
    ///
    /// The caller supplies the contents, for example from an embedded
    /// default or from a test fixture. The loader stores them and parses
    /// them as TOML when it loads. Where the caller got the contents is
    /// unknown to the crate.
    ConfigurationContents
);
