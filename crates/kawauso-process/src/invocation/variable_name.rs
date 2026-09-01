//! The name of an environment variable of an external command

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// The name of an environment variable of an external command
///
/// The value holds the name as the caller wrote it, and the crate checks
/// nothing. The rules for a name differ between the platforms, and the
/// operating system applies them when the command starts. A name that no
/// program reads is therefore a valid name, and a name that the platform
/// refuses, such as one that holds a NUL byte, fails when the command starts.
///
/// The value is an [`OsString`], because the environment of a program holds
/// the strings of the operating system and not the strings of Rust.
///
/// # Examples
///
/// ```
/// use kawauso_process::invocation::VariableName;
///
/// let name = VariableName::from("RUSTUP_TOOLCHAIN");
///
/// assert_eq!(name.get(), "RUSTUP_TOOLCHAIN");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VariableName(OsString);

impl VariableName {
    /// Creates a name from anything that becomes an [`OsString`]
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::invocation::VariableName;
    ///
    /// let name = VariableName::new("CARGO_TERM_COLOR");
    ///
    /// assert_eq!(name.get(), "CARGO_TERM_COLOR");
    /// ```
    pub fn new(name: impl Into<OsString>) -> Self {
        Self(name.into())
    }

    /// Returns the name as the caller wrote it
    pub fn get(&self) -> &OsStr {
        &self.0
    }
}

/// Shows the name for a reader
///
/// A name that is not valid UTF-8 shows the replacement character `U+FFFD`
/// for each byte that has no character. The text is for a person, and no
/// caller reads it back into a name.
impl Display for VariableName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0.to_string_lossy())
    }
}

/// Creates a name from a borrowed string, such as a literal
impl From<&str> for VariableName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// Creates a name from a string that the application built at runtime
impl From<String> for VariableName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

/// Creates a name from a borrowed string of the operating system
impl From<&OsStr> for VariableName {
    fn from(name: &OsStr) -> Self {
        Self::new(name)
    }
}

/// Creates a name from a string of the operating system
impl From<OsString> for VariableName {
    fn from(name: OsString) -> Self {
        Self::new(name)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that reads the name of a variable from the environment
    // holds an `OsString`, and the name takes it without a conversion of its
    // own.
    #[test]
    fn from_os_string_keeps_the_name() {
        let name = VariableName::from(OsString::from("RUSTUP_TOOLCHAIN"));

        assert_eq!(name.get(), "RUSTUP_TOOLCHAIN");
    }

    // The text of a name reaches a log line through `Display`, and a caller
    // that shows one name gets it as the caller wrote it.
    #[test]
    fn to_string_returns_the_name() {
        let name = VariableName::from("RUSTUP_TOOLCHAIN");

        assert_eq!(name.to_string(), "RUSTUP_TOOLCHAIN");
    }
}
