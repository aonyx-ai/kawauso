//! One argument of an external command

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;
use std::path::PathBuf;

/// One argument of an external command
///
/// An argument holds what the program receives, as the caller wrote it.
/// Nothing splits it at a space, removes a quotation mark, or expands a
/// character such as `*`, because no shell reads the command. An argument
/// that holds a space is one argument, and not two.
///
/// The value is an [`OsString`], because a program takes the strings of the
/// operating system and not the strings of Rust. An argument can therefore
/// carry a path that is not valid UTF-8, which a file system allows.
///
/// # Examples
///
/// ```
/// use kawauso_process::invocation::Argument;
///
/// let argument = Argument::from("--message=two words");
///
/// assert_eq!(argument.get(), "--message=two words");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Argument(OsString);

impl Argument {
    /// Creates an argument from anything that becomes an [`OsString`]
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::invocation::Argument;
    ///
    /// let argument = Argument::new("--short");
    ///
    /// assert_eq!(argument.get(), "--short");
    /// ```
    pub fn new(argument: impl Into<OsString>) -> Self {
        Self(argument.into())
    }

    /// Returns the argument as the caller wrote it
    pub fn get(&self) -> &OsStr {
        &self.0
    }
}

/// Shows the argument for a reader
///
/// An argument that is not valid UTF-8 shows the replacement character
/// `U+FFFD` for each byte that has no character. The text is for a person,
/// and no caller reads it back into an argument.
impl Display for Argument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0.to_string_lossy())
    }
}

/// Creates an argument from a borrowed string, such as a literal
impl From<&str> for Argument {
    fn from(argument: &str) -> Self {
        Self::new(argument)
    }
}

/// Creates an argument from a string that the application built at runtime
impl From<String> for Argument {
    fn from(argument: String) -> Self {
        Self::new(argument)
    }
}

/// Creates an argument from a borrowed string of the operating system
impl From<&OsStr> for Argument {
    fn from(argument: &OsStr) -> Self {
        Self::new(argument)
    }
}

/// Creates an argument from a string of the operating system
impl From<OsString> for Argument {
    fn from(argument: OsString) -> Self {
        Self::new(argument)
    }
}

/// Creates an argument from a borrowed path, such as the root of a project
impl From<&Path> for Argument {
    fn from(argument: &Path) -> Self {
        Self::new(argument)
    }
}

/// Creates an argument from a path that the application built at runtime
impl From<PathBuf> for Argument {
    fn from(argument: PathBuf) -> Self {
        Self::new(argument)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // An application that reads an argument from the environment holds an
    // `OsString`, and the argument takes it without a conversion of its own.
    #[test]
    fn from_os_string_keeps_the_string() {
        let argument = Argument::from(OsString::from("--short"));

        assert_eq!(argument.get(), "--short");
    }

    // An application that names a file for a command holds a path, and the
    // argument takes it without a conversion of its own.
    #[test]
    fn from_path_keeps_the_path() {
        let argument = Argument::from(Path::new("src/main.rs"));

        assert_eq!(argument.get(), "src/main.rs");
    }

    // The text of an argument reaches a log line through `Display`, and a
    // caller that shows one argument gets it without quotation marks.
    #[test]
    fn to_string_returns_the_argument() {
        let argument = Argument::from("two words");

        assert_eq!(argument.to_string(), "two words");
    }
}
