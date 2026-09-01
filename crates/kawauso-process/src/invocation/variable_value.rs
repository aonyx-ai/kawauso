//! The value of an environment variable of an external command

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;
use std::path::PathBuf;

/// The value of an environment variable of an external command
///
/// The value holds what the program reads, as the caller wrote it. Nothing
/// splits it at a space or expands a character such as `$`, because no shell
/// reads the command. A value that holds a space is one value.
///
/// The value is an [`OsString`], because the environment of a program holds
/// the strings of the operating system and not the strings of Rust. A value is
/// often a path, such as the directory that a build tool writes to, and a
/// path becomes a value without a conversion of its own.
///
/// # Examples
///
/// ```
/// use kawauso_process::invocation::VariableValue;
///
/// let value = VariableValue::from("-D warnings");
///
/// assert_eq!(value.get(), "-D warnings");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct VariableValue(OsString);

impl VariableValue {
    /// Creates a value from anything that becomes an [`OsString`]
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::invocation::VariableValue;
    ///
    /// let value = VariableValue::new("always");
    ///
    /// assert_eq!(value.get(), "always");
    /// ```
    pub fn new(value: impl Into<OsString>) -> Self {
        Self(value.into())
    }

    /// Returns the value as the caller wrote it
    pub fn get(&self) -> &OsStr {
        &self.0
    }
}

/// Shows the value for a reader
///
/// A value that is not valid UTF-8 shows the replacement character `U+FFFD`
/// for each byte that has no character. The text is for a person, and no
/// caller reads it back into a value.
impl Display for VariableValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0.to_string_lossy())
    }
}

/// Creates a value from a borrowed string, such as a literal
impl From<&str> for VariableValue {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Creates a value from a string that the application built at runtime
impl From<String> for VariableValue {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Creates a value from a borrowed string of the operating system
impl From<&OsStr> for VariableValue {
    fn from(value: &OsStr) -> Self {
        Self::new(value)
    }
}

/// Creates a value from a string of the operating system
impl From<OsString> for VariableValue {
    fn from(value: OsString) -> Self {
        Self::new(value)
    }
}

/// Creates a value from a borrowed path, such as the root of a project
impl From<&Path> for VariableValue {
    fn from(value: &Path) -> Self {
        Self::new(value)
    }
}

/// Creates a value from a path that the application built at runtime
impl From<PathBuf> for VariableValue {
    fn from(value: PathBuf) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that reads a value from the environment holds an
    // `OsString`, and the value takes it without a conversion of its own.
    #[test]
    fn from_os_string_keeps_the_value() {
        let value = VariableValue::from(OsString::from("always"));

        assert_eq!(value.get(), "always");
    }

    // An application that names a directory for a command holds a path, and
    // the value takes it without a conversion of its own.
    #[test]
    fn from_path_keeps_the_path() {
        let value = VariableValue::from(Path::new("target/coverage"));

        assert_eq!(value.get(), "target/coverage");
    }

    // The text of a value reaches a log line through `Display`, and a caller
    // that shows one value gets it without quotation marks.
    #[test]
    fn to_string_returns_the_value() {
        let value = VariableValue::from("-D warnings");

        assert_eq!(value.to_string(), "-D warnings");
    }
}
