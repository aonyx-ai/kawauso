//! The bytes that a command wrote to one of its streams

use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// The bytes that a command wrote to one of its streams
///
/// The value holds the bytes as the command wrote them. A command writes what
/// it wants. It can write the lines of a report, the bytes of an archive, or a
/// sequence that is not valid UTF-8 at all. A conversion into text would lose the bytes
/// that have no character. The type therefore keeps the bytes and offers the
/// text to the caller that wants it.
///
/// # Examples
///
/// ```
/// use kawauso_process::execution::Capture;
///
/// let capture = Capture::new("hello\n");
///
/// assert_eq!(capture.to_string_lossy().trim(), "hello");
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Capture(Vec<u8>);

impl Capture {
    /// Creates the capture of a stream from the bytes that a command wrote
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Capture;
    ///
    /// let capture = Capture::new(vec![b'h', b'i']);
    ///
    /// assert_eq!(capture.get(), b"hi");
    /// ```
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// Returns the bytes as the command wrote them
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Capture;
    ///
    /// let capture = Capture::new(b"hello");
    ///
    /// assert_eq!(capture.get(), b"hello");
    /// ```
    pub fn get(&self) -> &[u8] {
        &self.0
    }

    /// Returns whether the command wrote nothing to the stream
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Capture;
    ///
    /// let capture = Capture::new("");
    ///
    /// assert!(capture.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the capture as text
    ///
    /// A byte that is part of no valid character becomes the replacement
    /// character `U+FFFD`. The text is therefore always valid UTF-8, and a
    /// caller that needs the bytes of the command asks for [`get`][get]
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::execution::Capture;
    ///
    /// let capture = Capture::new(vec![0xff]);
    ///
    /// assert_eq!(capture.to_string_lossy(), "\u{fffd}");
    /// ```
    ///
    /// [get]: Capture::get
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }
}

/// Shows the capture for a reader
///
/// The text is the lossy conversion that [`to_string_lossy`][to-string-lossy]
/// returns, because a log line and an error message take text and not bytes.
///
/// [to-string-lossy]: Capture::to_string_lossy
impl Display for Capture {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str(&self.to_string_lossy())
    }
}

/// Creates the capture of a stream from the bytes that a command wrote
impl From<Vec<u8>> for Capture {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::missing_panics_doc,
        reason = "An assertion in a test panics by design. A `# Panics` section on every test would repeat that and give the reader no information."
    )]

    use super::*;

    /// Shows that `Capture` is Send and Sync
    ///
    /// A caller keeps the capture of a run in a report that another thread reads.
    /// This test holds the type to the auto traits that make this possible. The
    /// test exists because a private field of a later version could take them
    /// away without a word from the compiler.
    #[test]
    fn capture_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Capture>();
    }

    #[test]
    fn capture_is_send_sync_and_unpin() {
        fn assert_send_sync_and_unpin<T: Send + Sync + Unpin>() {}

        assert_send_sync_and_unpin::<Capture>();
    }

    /// Shows that a command that writes nothing leaves an empty capture
    ///
    /// A command that writes nothing leaves an empty capture, and a caller that
    /// reports only what a command said asks for this state.
    #[test]
    fn is_empty_without_bytes_reports_an_empty_capture() {
        let capture = Capture::default();

        assert!(capture.is_empty());
    }

    /// Shows that the text of the capture reaches a log line through `Display`
    ///
    /// The text of the capture reaches a log line through `Display`, and a caller
    /// that shows it gets the same text as the lossy conversion.
    #[test]
    fn to_string_returns_the_capture_as_text() {
        let capture = Capture::new("hello");

        assert_eq!(capture.to_string(), "hello");
    }
}
