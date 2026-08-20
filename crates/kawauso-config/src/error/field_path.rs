//! The place of a failure in the structure of a configuration file

use std::fmt::{Display, Formatter, Result};

/// The place of a value in the structure of a configuration file
///
/// A field path shows the keys that lead from the top of a configuration file
/// to one value. A dot divides the keys, as in `server.port`. An index in
/// square brackets comes after the key of an array of tables, as in
/// `peers[1].name`. One dot alone is the path of the file itself, and it
/// shows that the failure applies to the full document.
///
/// A field path is a report for a person. A path shows a user which entry to
/// correct. A path does not get a value from the file.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FieldPath(String);

impl FieldPath {
    /// Creates a field path from the keys that lead to a value
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path as text
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FieldPath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, formatter)
    }
}
