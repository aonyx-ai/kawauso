//! The path of a value in a configuration file

use std::fmt::{Display, Formatter, Result};

/// The path from the root of a document to one value, such as `server.port`
///
/// A path joins the keys that lead to a value with dots. The key of an array
/// of tables is followed by the index of the table in square brackets, as in
/// `peers[1].name`. The root of the document itself has no keys and is
/// written as a single dot.
///
/// A path identifies a value for a human reader. It is not a query: the
/// crate offers no way to look a path up in a document.
///
/// # Examples
///
/// ```
/// let path = kawauso_config::error::deserialize::FieldPath::new("server.port");
/// assert_eq!(path.to_string(), "server.port");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FieldPath(String);

impl FieldPath {
    /// Creates a field path from its text
    ///
    /// The text is stored as given; the function does not parse or validate
    /// it.
    ///
    /// # Examples
    ///
    /// ```
    /// let path = kawauso_config::error::deserialize::FieldPath::new("server.port");
    /// assert_eq!(path.as_str(), "server.port");
    /// ```
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path as text
    ///
    /// # Examples
    ///
    /// ```
    /// let path = kawauso_config::error::deserialize::FieldPath::new("server.port");
    /// assert_eq!(path.as_str(), "server.port");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Formats the path as its text, without quotes or other decoration
impl Display for FieldPath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    #[test]
    fn field_path_is_send_sync_and_unpin() {
        fn assert_send_sync_unpin<T: Send + Sync + Unpin>() {}
        assert_send_sync_unpin::<FieldPath>();
    }

    #[test]
    fn trait_display() {
        let path = FieldPath::new("server.port");

        let display = path.to_string();

        assert_eq!(display, "server.port");
    }
}
