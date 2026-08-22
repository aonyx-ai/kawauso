//! The locations that a search for a configuration file reads

use std::fmt::{Display, Formatter, Result};

use crate::loader::ConfigurationPath;

/// The locations that a search reads, in the order of the search
///
/// A location is a full path, and not a directory, because a user who has to
/// create the file needs its name as well. The order is the order of the
/// search, so the first location is the one that wins when more than one
/// holds a file.
///
/// The list exists for the report that a failed search produces. It is not a
/// record of what the file system holds: a location in it can hold a file, a
/// directory, or nothing at all.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SearchedLocations(Vec<ConfigurationPath>);

impl SearchedLocations {
    /// Creates a list of locations from the paths in the order of the search
    ///
    /// The order is preserved as given, because it decides which location
    /// wins and in which order a report names them.
    pub fn new(locations: impl Into<Vec<ConfigurationPath>>) -> Self {
        Self(locations.into())
    }

    /// Returns the locations in the order of the search
    pub fn as_slice(&self) -> &[ConfigurationPath] {
        &self.0
    }
}

/// Formats the locations as a list, with each path in backticks
impl Display for SearchedLocations {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        for (index, location) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }

            write!(formatter, "`{location}`")?;
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
        let locations = SearchedLocations::new(vec![
            ConfigurationPath::from("/otter/kawauso.toml"),
            ConfigurationPath::from("/kawauso.toml"),
        ]);

        let display = locations.to_string();

        assert_eq!(display, "`/otter/kawauso.toml`, `/kawauso.toml`");
    }
}
