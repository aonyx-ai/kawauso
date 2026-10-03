//! The identifier of a project

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use super::portable_name::is_portable;
use crate::error::ParseProjectIdentifierError;

/// The identifier of a project
///
/// The identifier names the data directory of a project. Two checkouts with
/// the same identifier share one data directory, so the application
/// supplies a value that stays the same for clones, worktrees, and renamed
/// checkouts of one project. A value in the configuration file that the
/// users of the application commit is an example. A project that changes its
/// identifier gets a new, empty data directory.
///
/// The identifier becomes a component of a path, so it holds from 1 to 255
/// ASCII letters, ASCII digits, and hyphens, and it is not the name of a
/// device on Windows, such as `CON` or `COM1`. These rules keep the data
/// directory below the directory of the application, also for a value from
/// a repository that the user did not write. The rules are the same on every
/// platform, because users of all platforms share such a file.
///
/// Every way to create an identifier checks the rules and can fail,
/// deserialization included. The crate never changes a value to make it
/// fit, because two values that become equal would share one directory.
///
/// Identifiers that differ only in case, such as `Example` and `example`,
/// share one directory on the default file systems of macOS and Windows,
/// and get two directories on Linux. An application that wants one directory
/// on every platform uses identifiers that differ in more than case.
///
/// # Examples
///
/// ```
/// use kawauso_project::project::ProjectIdentifier;
///
/// let identifier: ProjectIdentifier = "example".parse()?;
///
/// assert_eq!(identifier.get(), "example");
/// assert!("../example".parse::<ProjectIdentifier>().is_err());
/// # Ok::<(), kawauso_project::error::ParseProjectIdentifierError>(())
/// ```
///
/// An application that keeps the identifier in its configuration file
/// deserializes it with the rest of the file, and a malformed value fails the
/// load:
///
/// ```
/// use serde::Deserialize;
///
/// use kawauso_project::project::ProjectIdentifier;
///
/// #[derive(Deserialize)]
/// struct Configuration {
///     id: ProjectIdentifier,
/// }
///
/// let configuration: Configuration = toml::from_str(r#"id = "example""#)?;
///
/// assert_eq!(configuration.id.get(), "example");
/// assert!(toml::from_str::<Configuration>(r#"id = "../example""#).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct ProjectIdentifier(String);

impl ProjectIdentifier {
    /// Returns the identifier as a string
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl Display for ProjectIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// A derived implementation writes a newtype struct, which a format such as
// RON keeps, while deserialization reads a string. The identifier therefore
// writes the string itself, so that every format reads what it wrote.
impl Serialize for ProjectIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl FromStr for ProjectIdentifier {
    type Err = ParseProjectIdentifierError;

    fn from_str(identifier: &str) -> Result<Self, Self::Err> {
        Self::try_from(identifier.to_owned())
    }
}

impl TryFrom<&str> for ProjectIdentifier {
    type Error = ParseProjectIdentifierError;

    fn try_from(identifier: &str) -> Result<Self, Self::Error> {
        Self::try_from(identifier.to_owned())
    }
}

impl TryFrom<String> for ProjectIdentifier {
    type Error = ParseProjectIdentifierError;

    // Deserialization goes through this function as well, which the
    // attribute of serde on the type selects.
    // project[impl name.deserialize]
    // project[impl name.error]
    // project[impl name.identifier]
    fn try_from(identifier: String) -> Result<Self, Self::Error> {
        if is_portable(&identifier) {
            Ok(Self(identifier))
        } else {
            Err(ParseProjectIdentifierError::MalformedIdentifier { identifier })
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    /// A document with one identifier, for the tests of deserialization
    #[derive(Debug, Deserialize)]
    struct Document {
        /// The identifier of a project
        id: ProjectIdentifier,
    }

    // project[verify name.deserialize]
    #[test]
    fn deserialize_with_a_malformed_identifier_returns_an_error() {
        let result = toml::from_str::<Document>(r#"id = "../example""#);

        assert!(result.is_err());
    }

    // project[verify name.deserialize]
    #[test]
    fn deserialize_with_a_portable_identifier_returns_the_identifier() {
        let document: Document = toml::from_str(r#"id = "example""#).unwrap();

        assert_eq!(document.id.get(), "example");
    }

    // project[verify name.identifier]
    // project[verify name.error]
    #[test]
    fn from_str_with_a_malformed_identifier_returns_an_error() {
        let error = "C:".parse::<ProjectIdentifier>().unwrap_err();

        assert!(matches!(
            error,
            ParseProjectIdentifierError::MalformedIdentifier { .. }
        ));
    }

    // project[verify name.identifier]
    #[test]
    fn from_str_with_a_portable_identifier_returns_the_identifier() {
        let identifier: ProjectIdentifier = "example".parse().unwrap();

        assert_eq!(identifier.get(), "example");
    }

    // Deserialization reads a string, so serialization writes one as well,
    // and not a newtype struct that some formats keep.
    #[test]
    fn serialize_writes_a_string() {
        let identifier: ProjectIdentifier = "example".parse().unwrap();

        serde_test::assert_ser_tokens(&identifier, &[serde_test::Token::Str("example")]);
    }

    // project[verify name.error]
    #[test]
    fn to_string_of_the_error_escapes_the_value() {
        let error = "aux\u{1b}[2J".parse::<ProjectIdentifier>().unwrap_err();

        assert_eq!(
            error.to_string(),
            "the project identifier \"aux\\u{1b}[2J\" is malformed, because an identifier holds 1 to 255 ASCII letters, digits, and hyphens, and does not name a device on Windows"
        );
    }

    // project[verify name.identifier]
    // project[verify name.error]
    #[test]
    fn try_from_a_str_with_a_malformed_identifier_returns_an_error() {
        let error = ProjectIdentifier::try_from("/example").unwrap_err();

        assert!(matches!(
            error,
            ParseProjectIdentifierError::MalformedIdentifier { .. }
        ));
    }

    // project[verify name.identifier]
    // project[verify name.error]
    #[test]
    fn try_from_a_string_with_a_malformed_identifier_returns_an_error() {
        let error = ProjectIdentifier::try_from(String::from("example project")).unwrap_err();

        assert!(matches!(
            error,
            ParseProjectIdentifierError::MalformedIdentifier { .. }
        ));
    }
}
