//! A position in a configuration file

pub mod column;
pub mod line;

use std::fmt::{Display, Formatter, Result};

pub use self::column::Column;
pub use self::line::Line;

/// A line and a column in a configuration file
///
/// Both numbers count from one, the way editors display them. A reader can
/// therefore jump from an error message straight to the position without any
/// conversion.
///
/// The line and the column have distinct types, so the compiler rejects a
/// call that passes them in the wrong order.
///
/// # Examples
///
/// ```
/// let pos = kawauso_config::error::deserialize::Position::builder()
///     .line(kawauso_config::error::deserialize::position::Line::new(2))
///     .column(kawauso_config::error::deserialize::position::Column::new(8))
///     .build();
/// assert_eq!(pos.to_string(), "line 2, column 8");
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, bon::Builder)]
pub struct Position {
    /// The line of the position
    line: Line,

    /// The column of the position
    column: Column,
}

impl Position {
    /// Returns the column
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = kawauso_config::error::deserialize::Position::builder()
    ///     .line(kawauso_config::error::deserialize::position::Line::new(2))
    ///     .column(kawauso_config::error::deserialize::position::Column::new(8))
    ///     .build();
    /// let column = pos.column();
    /// assert_eq!(column.get(), 8);
    /// ```
    pub fn column(&self) -> Column {
        self.column
    }

    /// Returns the line
    ///
    /// # Examples
    ///
    /// ```
    /// let pos = kawauso_config::error::deserialize::Position::builder()
    ///     .line(kawauso_config::error::deserialize::position::Line::new(2))
    ///     .column(kawauso_config::error::deserialize::position::Column::new(8))
    ///     .build();
    /// let line = pos.line();
    /// assert_eq!(line.get(), 2);
    /// ```
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
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    #[test]
    fn position_is_send_sync_and_unpin() {
        fn assert_send_sync_unpin<T: Send + Sync + Unpin>() {}
        assert_send_sync_unpin::<Position>();
    }

    #[test]
    fn trait_display() {
        let position = Position::builder()
            .line(Line::new(2))
            .column(Column::new(8))
            .build();

        let display = position.to_string();

        assert_eq!(display, "line 2, column 8");
    }
}
