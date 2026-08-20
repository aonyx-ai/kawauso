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
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FieldPath(String);

impl FieldPath {
    /// Creates a field path from its text
    ///
    /// The text is stored as given; the function does not parse or validate
    /// it.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path as text
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
