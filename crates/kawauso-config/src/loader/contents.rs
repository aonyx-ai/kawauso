//! The contents of a configuration file
//!
//! A caller never names the contents of a configuration. A constructor of the
//! loader takes what they wrap, and no error reports them. The module is
//! therefore private, which keeps it out of the public API and leaves the
//! crate free to change it.

use typed_fields::name;

name!(
    /// The contents of a configuration file
    ///
    /// The caller supplies the contents, for example from an embedded
    /// default or from a test fixture. The loader stores them and parses
    /// them as TOML when it loads. Where the caller got the contents is
    /// unknown to the crate.
    Contents
);
