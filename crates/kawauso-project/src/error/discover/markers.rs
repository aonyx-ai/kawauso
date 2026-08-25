//! The markers that a search tests

use std::fmt::{Display, Formatter, Result};

use crate::search::Marker;

/// The markers that a search tests, in the order of the test
///
/// The list exists for the report that a failed search produces. The order
/// is the order of the test, so the first marker is the one that wins when
/// more than one exists in a directory. The configuration file of the
/// application is the first marker, and the markers that the developer
/// named follow it.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Markers(Vec<Marker>);

impl Markers {
    /// Creates a list of markers in the order of the test
    ///
    /// The order is preserved as given, because it decides which marker
    /// wins and in which order a report names them.
    pub fn new(markers: impl Into<Vec<Marker>>) -> Self {
        Self(markers.into())
    }

    /// Returns the markers in the order of the test
    pub fn as_slice(&self) -> &[Marker] {
        &self.0
    }
}

/// Formats the markers as a list, with each path in backticks
impl Display for Markers {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        for (index, marker) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }

            write!(formatter, "`{marker}`")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    #[test]
    fn trait_display() {
        let markers = Markers::new(vec![
            Marker::from(".config/kawauso.toml"),
            Marker::from(".git"),
        ]);

        let display = markers.to_string();

        assert_eq!(display, "`.config/kawauso.toml`, `.git`");
    }
}
