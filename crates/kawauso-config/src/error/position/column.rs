//! The column of a position in a configuration file

use typed_fields::number;

number!(
    /// The column of a position in a configuration file
    ///
    /// Columns count characters rather than bytes, and they count from one,
    /// the way editors display them.
    Column,
    usize
);
