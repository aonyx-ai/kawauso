//! One line of the output of a command

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use super::stream::Stream;
use super::text::Text;

/// One line of the output of a command
///
/// A line carries the text that the command wrote and the stream that wrote
/// it. The text holds neither the `\n` nor the `\r\n` that ended the line, so
/// a caller can put the line into a log, a report, or an event of its own
/// without a trim.
///
/// The last line of a stream can end with no such characters at all, when the
/// command ends after its last byte. That line reaches the caller as well.
///
/// # Examples
///
/// ```
/// use kawauso_process::run::Line;
/// use kawauso_process::run::Stream;
///
/// let line = Line::new(Stream::StandardError, "no such file");
///
/// assert_eq!(line.text().get(), "no such file");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Line {
    /// The stream that produced the line
    stream: Stream,

    /// The text of the line, without the characters that ended it
    text: Text,
}

impl Line {
    /// Creates one line of the output of a command
    ///
    /// A run builds this value, and a caller builds one where a test stands
    /// in for a command that no one starts.
    pub fn new(stream: Stream, text: impl Into<Text>) -> Self {
        Self {
            stream,
            text: text.into(),
        }
    }

    /// Returns the stream that produced the line
    ///
    /// A caller that shows a diagnostic apart from a result, or that drops
    /// one of the two, reads the stream here.
    // process[impl stream.tag]
    pub fn stream(&self) -> Stream {
        self.stream
    }

    /// Returns the text of the line
    pub fn text(&self) -> &Text {
        &self.text
    }
}

/// Shows the line for a reader
///
/// The text is the line alone, without the name of the stream, because a
/// caller that wants the name has the stream and chooses how to show it.
impl Display for Line {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str(self.text.get())
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
    fn line_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Line>();
    }

    // The text of a line reaches a log line through `Display`, and a caller
    // that shows one line gets the text without the name of the stream.
    #[test]
    fn to_string_returns_the_text_of_the_line() {
        let line = Line::new(Stream::StandardOutput, "hello");

        assert_eq!(line.to_string(), "hello");
    }
}
