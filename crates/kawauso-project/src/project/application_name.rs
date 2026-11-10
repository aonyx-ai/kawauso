//! The name of an application

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use super::portable_name::is_portable;
use crate::error::ParseApplicationNameError;

/// The name of an application
///
/// The name identifies the application whose project this is. The crate
/// derives the conventional location of the configuration file from it,
/// `.config/<name>.toml` inside the project, and the data directory of the
/// project, `<data>/<name>/projects/<identifier>` in the local data directory
/// of the user.
///
/// Use the name that a user types to start the application, so that the
/// user finds the files under a name that they recognize.
///
/// The name becomes a component of a path, so it holds from 1 to 250 ASCII
/// letters, ASCII digits, and hyphens. It is also not a name that Windows
/// keeps for a device: `CON`, `PRN`, `AUX`, `NUL`, `COM0` to `COM9`, or
/// `LPT0` to `LPT9`, in upper case, lower case, or a mix of the two. The
/// rules are the same on every platform. Every way to create a name checks
/// these rules and can fail. A builder that takes a name therefore cannot
/// fail because of it.
///
/// The rules do not permit a dot or a space, because some file systems change
/// a name that holds them. Windows removes a trailing space or dot, for
/// example. The rules also do not permit letters outside ASCII, because macOS
/// compares the Unicode forms of a name as one name. The crate never changes
/// a value to make it valid. A later release can permit more characters, but
/// a name that is valid now stays valid, and its files keep their directory.
///
/// # Examples
///
/// ```
/// use kawauso_project::project::ApplicationName;
///
/// let name: ApplicationName = "example".parse()?;
///
/// assert_eq!(name.get(), "example");
/// assert!("example_app".parse::<ApplicationName>().is_err());
/// # Ok::<(), kawauso_project::error::ParseApplicationNameError>(())
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ApplicationName(String);

impl ApplicationName {
    /// Returns the name as a string
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Display for ApplicationName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ApplicationName {
    type Err = ParseApplicationNameError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::try_from(name.to_owned())
    }
}

impl TryFrom<&str> for ApplicationName {
    type Error = ParseApplicationNameError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        Self::try_from(name.to_owned())
    }
}

impl TryFrom<String> for ApplicationName {
    type Error = ParseApplicationNameError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if is_portable(&name) {
            Ok(Self(name))
        } else {
            Err(ParseApplicationNameError::MalformedName { name })
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn from_str_with_a_malformed_name_returns_an_error() {
        let error = "example/app".parse::<ApplicationName>().unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }

    #[test]
    fn from_str_with_a_portable_name_returns_the_name() {
        let name: ApplicationName = "example".parse().unwrap();

        assert_eq!(name.get(), "example");
    }

    #[test]
    fn to_string_of_the_error_escapes_the_value() {
        let error = "COM1\n".parse::<ApplicationName>().unwrap_err();

        assert_eq!(
            error.to_string(),
            "the application name \"COM1\\n\" is malformed, because a name holds 1 to 250 ASCII letters, digits, and hyphens, and does not name a device on Windows"
        );
    }

    #[test]
    fn try_from_a_str_with_a_malformed_name_returns_an_error() {
        let error = ApplicationName::try_from("").unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }

    #[test]
    fn try_from_a_string_with_a_malformed_name_returns_an_error() {
        let error = ApplicationName::try_from(String::from("..")).unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }
}
