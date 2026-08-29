//! The text of one line of output

use typed_fields::name;

name!(
    /// The text of one line of output
    ///
    /// The value holds the line without the characters that ended it. A
    /// command writes bytes, and a byte that is part of no valid character
    /// becomes the replacement character `U+FFFD`, so the text is always
    /// valid UTF-8. A caller that needs the bytes as the command wrote them
    /// reads them from the capture of the result.
    Text
);
