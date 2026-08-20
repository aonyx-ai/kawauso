//! The line of a position in a configuration file

use typed_fields::number;

number!(
    /// The line of a position in a configuration file
    ///
    /// Lines count from one, the way editors display them.
    Line,
    usize
);
