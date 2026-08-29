//! The output that a command wrote to one of its streams

use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// The output that a command wrote to one of its streams
///
/// The value holds the bytes as the command wrote them. A command writes what
/// it wants: the lines of a report, the bytes of an archive, or a sequence
/// that is not valid UTF-8 at all. A conversion into text would lose the
/// bytes that have no character, so the type keeps the bytes and offers the
/// text to the caller that wants it.
///
/// # Examples
///
/// ```
/// use kawauso_process::execution::Output;
///
/// let output = Output::new("hello\n");
///
/// assert_eq!(output.to_string_lossy().trim(), "hello");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Output(Vec<u8>);

impl Output {
    /// Creates the output of a stream from the bytes that a command wrote
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Output;
    ///
    /// let output = Output::new(vec![b'h', b'i']);
    ///
    /// assert_eq!(output.get(), b"hi");
    /// ```
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// Returns the bytes as the command wrote them
    pub fn get(&self) -> &[u8] {
        &self.0
    }

    /// Returns whether the command wrote nothing to the stream
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the output as text
    ///
    /// A byte that is part of no valid character becomes the replacement
    /// character `U+FFFD`. The text is therefore always valid UTF-8, and a
    /// caller that needs the bytes of the command asks for [`get`][get]
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Output;
    ///
    /// let output = Output::new(vec![0xff]);
    ///
    /// assert_eq!(output.to_string_lossy(), "\u{fffd}");
    /// ```
    ///
    /// [get]: Output::get
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }
}

/// Shows the output for a reader
///
/// The text is the lossy conversion that [`to_string_lossy`][to-string-lossy]
/// returns, because a log line and an error message take text and not bytes.
///
/// [to-string-lossy]: Output::to_string_lossy
impl Display for Output {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str(&self.to_string_lossy())
    }
}

/// Creates the output of a stream from the bytes of a capture
impl From<Vec<u8>> for Output {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller keeps the output of a run in a report that another thread
    // reads. This test holds the type to the auto traits that make this
    // possible, because a private field of a later version could take them
    // away without a word from the compiler.
    #[test]
    fn output_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Output>();
    }

    // A command that writes nothing leaves an empty capture, and a caller
    // that reports only what a command said asks for this state.
    #[test]
    fn is_empty_without_bytes_reports_an_empty_output() {
        let output = Output::default();

        assert!(output.is_empty());
    }

    // The text of the output reaches a log line through `Display`, and a
    // caller that shows it gets the same text as the lossy conversion.
    #[test]
    fn to_string_returns_the_output_as_text() {
        let output = Output::new("hello");

        assert_eq!(output.to_string(), "hello");
    }
}
