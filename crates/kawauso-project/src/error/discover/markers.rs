//! The markers that a search tests

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use crate::search::Marker;

/// The markers that a search tests, in the order of the test
///
/// The list exists for the report that a failed search produces. A plain
/// vector cannot carry that report, because it has no [`Display`]
/// implementation, and the message of an error needs one.
///
/// The order is the order of the test. The first marker is therefore the one
/// that wins when a directory holds more than one of them.
///
/// # Examples
///
/// ```
/// use kawauso_project::error::discover::Markers;
/// use kawauso_project::search::Marker;
///
/// let markers = Markers::new(vec![Marker::from(".git"), Marker::from("Cargo.toml")]);
///
/// assert_eq!(markers.to_string(), "`.git`, `Cargo.toml`");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Markers(Vec<Marker>);

impl Markers {
    /// Creates a list of markers in the order of the test
    ///
    /// The order is kept as it is given, because it decides which marker wins
    /// and in which order a report names them.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::error::discover::Markers;
    /// use kawauso_project::search::Marker;
    ///
    /// let markers = Markers::new(vec![Marker::from(".git")]);
    ///
    /// assert_eq!(markers.as_slice().len(), 1);
    /// ```
    pub fn new(markers: impl Into<Vec<Marker>>) -> Self {
        Self(markers.into())
    }

    /// Returns the markers in the order of the test
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_project::error::discover::Markers;
    /// use kawauso_project::search::Marker;
    ///
    /// let markers = Markers::new(vec![Marker::from(".git"), Marker::from("Cargo.toml")]);
    ///
    /// assert_eq!(markers.as_slice()[1], Marker::from("Cargo.toml"));
    /// ```
    pub fn as_slice(&self) -> &[Marker] {
        &self.0
    }
}

/// Formats the markers as a list, with each path in backticks
impl Display for Markers {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let markers: Vec<String> = self.0.iter().map(|marker| format!("`{marker}`")).collect();

        formatter.write_str(&markers.join(", "))
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
    fn markers_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<Markers>();
    }

    #[test]
    fn trait_display() {
        let markers = Markers::new(vec![Marker::from(".git"), Marker::from("src/main.rs")]);

        let display = markers.to_string();

        assert_eq!(display, "`.git`, `src/main.rs`");
    }
}
