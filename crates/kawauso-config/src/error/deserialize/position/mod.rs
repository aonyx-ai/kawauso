//! A position in a configuration file

pub mod column;
pub mod line;

use std::fmt::{Display, Formatter, Result};

pub use self::column::Column;
pub use self::line::Line;

/// A line and a column in a configuration file
///
/// Both numbers count from one, the way editors display them, so a reader
/// can jump from an error message straight to the position without any
/// conversion.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Position {
    /// The line of the position
    line: Line,

    /// The column of the position
    column: Column,
}

impl Position {
    /// Creates a position from a line and a column
    ///
    /// The two arguments have distinct types, so the compiler rejects a call
    /// that passes them in the wrong order.
    pub fn new(line: Line, column: Column) -> Self {
        Self { line, column }
    }

    /// Returns the column
    pub fn column(&self) -> Column {
        self.column
    }

    /// Returns the line
    pub fn line(&self) -> Line {
        self.line
    }
}

/// Formats the position as `line 2, column 8`
impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "line {}, column {}", self.line, self.column)
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
        let position = Position::new(Line::new(2), Column::new(8));

        let display = position.to_string();

        assert_eq!(display, "line 2, column 8");
    }
}
