//! The stream of a command that produced a line of output

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// The stream of a command that produced a line of output
///
/// A command writes its result to one stream and its diagnostics to another,
/// and a reader of the output has to know which of the two it reads. A line
/// carries this value, so that a caller can show a diagnostic apart from a
/// result, or drop one of the two.
///
/// A command has these two output streams and no other, so a match on the
/// value needs no arm for a stream that a later release adds.
///
/// # Examples
///
/// ```
/// use kawauso_process::run::Stream;
///
/// assert_eq!(Stream::StandardError.to_string(), "standard error");
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Stream {
    /// The stream on which a command states why it stopped
    StandardError,

    /// The stream on which a command writes its result
    StandardOutput,
}

/// Names the stream for a reader
///
/// The name is the one that the documentation of an operating system uses, so
/// that a log line names the stream that a reader looks for.
impl Display for Stream {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::StandardError => formatter.write_str("standard error"),
            Self::StandardOutput => formatter.write_str("standard output"),
        }
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller keeps a line in a report that another thread reads. This test
    // holds the type to the auto traits that make this possible, because a
    // private field of a later version could take them away without a word
    // from the compiler.
    #[test]
    fn stream_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Stream>();
    }

    // A log line names the stream through `Display`, and the name of the
    // standard error is the one that a reader of a manual knows.
    #[test]
    fn to_string_of_the_standard_error_names_the_stream() {
        let stream = Stream::StandardError;

        assert_eq!(stream.to_string(), "standard error");
    }

    // A log line names the stream through `Display`, and the name of the
    // standard output is the one that a reader of a manual knows.
    #[test]
    fn to_string_of_the_standard_output_names_the_stream() {
        let stream = Stream::StandardOutput;

        assert_eq!(stream.to_string(), "standard output");
    }
}
