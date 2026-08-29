//! The description of one external command
//!
//! An invocation names the program of a command, the arguments of the
//! program, and the directory in which the command runs. This module holds
//! the invocation and the types of its parts.
//!
//! An invocation is a value, and nothing starts a program when an application
//! builds one. An application can therefore describe a command once, write
//! the command to a log, and name it in an error.

pub mod argument;
pub mod program;
pub mod working_directory;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

pub use self::argument::Argument;
pub use self::program::Program;
pub use self::working_directory::WorkingDirectory;

/// The description of one external command
///
/// An invocation holds the program that the command starts, the arguments of
/// the program, and the directory in which the command runs. The arguments
/// keep the order in which the caller named them, and the working directory
/// is optional.
///
/// No shell reads the command. Nothing splits an argument at a space, removes
/// a quotation mark, or expands a character such as `*`, so the program
/// receives every argument as the caller wrote it. An application that wants
/// a shell names the shell as the program, and gives the command line to it
/// as one argument.
///
/// The type renders as a command line through [`Display`], which gives a log
/// line or an error message the name of the command.
///
/// # Examples
///
/// A command with two arguments:
///
/// ```
/// use kawauso_process::Invocation;
///
/// let invocation = Invocation::new("git").arg("status").arg("--short");
///
/// assert_eq!(invocation.to_string(), "git status --short");
/// ```
///
/// A command that runs in a directory of its own:
///
/// ```
/// use std::path::Path;
///
/// use kawauso_process::Invocation;
/// use kawauso_process::invocation::WorkingDirectory;
///
/// let invocation = Invocation::new("cargo")
///     .arg("build")
///     .in_directory("crates/example");
///
/// assert_eq!(
///     invocation.working_directory().map(WorkingDirectory::get),
///     Some(Path::new("crates/example"))
/// );
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Invocation {
    /// The program that the command starts
    program: Program,

    /// The arguments that the program receives
    ///
    /// The order is the order in which the caller named them, because it is
    /// the order in which the program receives them.
    arguments: Vec<Argument>,

    /// The directory in which the command runs
    ///
    /// `None` when the caller named no directory. The command then runs where
    /// the process runs.
    working_directory: Option<WorkingDirectory>,
}

impl Invocation {
    /// Describes a command that starts the program that the caller names
    ///
    /// The program is the path of an executable file, or the bare name of
    /// one. The command has no argument yet, and it runs where the process
    /// runs. [`arg`][arg] adds an argument, and
    /// [`in_directory`][in-directory] names a working directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("git");
    ///
    /// assert_eq!(invocation.program().get(), std::path::Path::new("git"));
    /// ```
    ///
    /// [arg]: Invocation::arg
    /// [in-directory]: Invocation::in_directory
    // process[impl invocation.program]
    pub fn new(program: impl Into<Program>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            working_directory: None,
        }
    }

    /// Adds one argument to the command
    ///
    /// Every call adds one argument, and the program receives the arguments
    /// in the order of the calls. The value reaches the program as the caller
    /// wrote it, so an argument that holds a space stays one argument.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("git")
    ///     .arg("commit")
    ///     .arg("--message=two words");
    ///
    /// assert_eq!(invocation.arguments().len(), 2);
    /// ```
    // process[impl invocation.arguments]
    pub fn arg(mut self, argument: impl Into<Argument>) -> Self {
        self.arguments.push(argument.into());

        self
    }

    /// Adds every argument of an iterator to the command
    ///
    /// The command keeps the order of the iterator, and a later call appends
    /// to the arguments that the command holds. Use this method for a caller
    /// that holds its arguments in a collection, and [`arg`][arg] for one
    /// that names them.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("cargo").args(["build", "--release"]);
    ///
    /// assert_eq!(invocation.to_string(), "cargo build --release");
    /// ```
    ///
    /// [arg]: Invocation::arg
    pub fn args(mut self, arguments: impl IntoIterator<Item = impl Into<Argument>>) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));

        self
    }

    /// Returns the arguments of the command, in the order of the calls
    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    /// Runs the command in the directory that the caller names
    ///
    /// Use this method for a command that works on the directory that it runs
    /// in, such as a build tool that reads the manifest of a project. A
    /// command without a working directory runs where the process runs.
    ///
    /// A later call replaces the directory of an earlier one, because a
    /// command runs in one directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("cargo")
    ///     .arg("test")
    ///     .in_directory("crates/example");
    ///
    /// assert!(invocation.working_directory().is_some());
    /// ```
    // process[impl invocation.directory]
    pub fn in_directory(mut self, directory: impl Into<WorkingDirectory>) -> Self {
        self.working_directory = Some(directory.into());

        self
    }

    /// Returns the program that the command starts
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Returns the directory in which the command runs
    ///
    /// `None` when the caller named no directory. The command then runs where
    /// the process runs.
    pub fn working_directory(&self) -> Option<&WorkingDirectory> {
        self.working_directory.as_ref()
    }
}

/// Renders the command as a command line for a reader
///
/// The line names the program, and then every argument in the order in which
/// the caller named them. It does not name the working directory, because a
/// command line holds the command and not the place that it runs in.
///
/// The line is for a person. No caller reads it back, and no shell runs it,
/// so it is not a command line that a shell has to accept.
// process[impl invocation.display]
impl Display for Invocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write_word(formatter, &self.program.get().to_string_lossy())?;

        for argument in &self.arguments {
            formatter.write_str(" ")?;
            write_word(formatter, &argument.get().to_string_lossy())?;
        }

        Ok(())
    }
}

/// Writes one word of a command line
///
/// A space separates the words of the line, so a word that holds a space
/// reads as two words, and a word that is empty reads as none. Such a word
/// goes between quotation marks, and a reader then sees where it starts and
/// where it ends.
///
/// The marks are for the reader. A word that holds a quotation mark of its own
/// keeps it, because no shell parses the line.
///
/// # Errors
///
/// Returns the error of the formatter when it cannot take the text.
fn write_word(formatter: &mut Formatter<'_>, word: &str) -> Result {
    if word.is_empty() || word.contains(char::is_whitespace) {
        write!(formatter, "\"{word}\"")
    } else {
        formatter.write_str(word)
    }
}

#[cfg(test)]
mod tests {
    // An assertion in a test panics by design. A `# Panics` section on every
    // test would repeat that and give the reader no information.
    #![allow(clippy::missing_panics_doc)]

    use super::*;

    // A caller keeps an invocation in a field of a type that another thread
    // reads. This test holds the invocation to the auto traits that make this
    // possible, because a private field of a later version could take them
    // away without a word from the compiler.
    #[test]
    fn invocation_is_send_and_sync() {
        fn assert_send_and_sync<T: Send + Sync>() {}

        assert_send_and_sync::<Invocation>();
    }
}
