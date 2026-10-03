//! The name of an application that has a configuration file

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use super::portable_name::is_portable;
use crate::error::ParseApplicationNameError;

/// The name of an application that has a configuration file
///
/// A search derives the name of the file from the name of the application,
/// so that the developer supplies one word and gets the same file name on
/// every platform. A search near the working directory looks for
/// `<name>.toml`, and a search in the directory of the user looks for
/// `config.toml` in a directory `<name>`.
///
/// Use the name that a user types to start the application, so that the
/// user finds the files under a name that they recognize.
///
/// The name becomes part of a path, so it holds from 1 to 250 ASCII
/// letters, ASCII digits, and hyphens, and it is not the name of a device on
/// Windows, such as `CON` or `COM1`. Every way to create a name checks these
/// rules and can fail. A constructor of the loader that takes a name
/// therefore cannot fail because of it.
///
/// # Examples
///
/// ```
/// use kawauso_config::ApplicationName;
///
/// let name: ApplicationName = "example".parse()?;
///
/// assert_eq!(name.get(), "example");
/// assert!("example_app".parse::<ApplicationName>().is_err());
/// # Ok::<(), kawauso_config::error::ParseApplicationNameError>(())
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

    // config[impl name.application]
    // config[impl name.error]
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

    // The rules of a name hold more than the set of characters. A name of a
    // device on Windows obeys that set, so only the rule on devices rejects
    // it.
    // config[verify name.application]
    // config[verify name.error]
    #[test]
    fn from_str_with_a_device_returns_an_error() {
        let error = "CON".parse::<ApplicationName>().unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }

    // config[verify name.application]
    // config[verify name.error]
    #[test]
    fn from_str_with_a_malformed_name_returns_an_error() {
        let error = "example/app".parse::<ApplicationName>().unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }

    // config[verify name.application]
    #[test]
    fn from_str_with_a_portable_name_returns_the_name() {
        let name: ApplicationName = "example".parse().unwrap();

        assert_eq!(name.get(), "example");
    }

    // config[verify name.error]
    #[test]
    fn to_string_of_the_error_escapes_the_value() {
        let error = "COM1\n".parse::<ApplicationName>().unwrap_err();

        assert_eq!(
            error.to_string(),
            "the application name \"COM1\\n\" is malformed, because a name holds 1 to 250 ASCII letters, digits, and hyphens, and does not name a device on Windows"
        );
    }

    // config[verify name.application]
    // config[verify name.error]
    #[test]
    fn try_from_a_str_with_a_malformed_name_returns_an_error() {
        let error = ApplicationName::try_from("").unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }

    // config[verify name.application]
    // config[verify name.error]
    #[test]
    fn try_from_a_string_with_a_malformed_name_returns_an_error() {
        let error = ApplicationName::try_from(String::from("..")).unwrap_err();

        assert!(matches!(
            error,
            ParseApplicationNameError::MalformedName { .. }
        ));
    }
}
