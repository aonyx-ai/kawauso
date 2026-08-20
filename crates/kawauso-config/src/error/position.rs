//! The place of a failure in a configuration file

use std::fmt::{Display, Formatter, Result};

/// A place in a configuration file
///
/// A position points to one character in a configuration file. The line and
/// the column start at one, as an editor shows them. A reader can thus go to
/// the position immediately.
///
/// A position is a report for a person. Use a position to show a user where
/// to look. Do not use a position to divide a file into parts.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Position {
    /// Creates a position from a line and a column
    ///
    /// The line and the column start at one. If a caller has a count that
    /// starts at zero, the caller must add one to that count first.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Returns the column, counted from one
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns the line, counted from one
    pub fn line(&self) -> usize {
        self.line
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "line {}, column {}", self.line, self.column)
    }
}
