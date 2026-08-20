//! A position in a configuration file

use std::fmt::{Display, Formatter, Result};

/// A line and a column in a configuration file
///
/// Both numbers count from one, the way editors display them, so a reader
/// can jump from an error message straight to the position without any
/// conversion.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Position {
    /// Creates a position from a line and a column
    ///
    /// Both arguments count from one. A caller that has zero-based numbers
    /// must add one to them first.
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

/// Formats the position as `line 2, column 8`
impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "line {}, column {}", self.line, self.column)
    }
}
