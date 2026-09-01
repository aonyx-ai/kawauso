//! The description of one external command
//!
//! An invocation names the program of a command, the arguments of the
//! program, and the directory in which the command runs. It also carries the
//! variables that the caller sets in the environment of the command. This
//! module holds the invocation and the types of its parts.
//!
//! An invocation is a value, and nothing starts a program when an application
//! builds one. An application can therefore describe a command once, write
//! the command to a log, and name it in an error.
//!
//! The [run][run] method starts the command that the invocation describes,
//! and it reports what the command produced. The [start][start] method starts
//! the same command and gives its output to the caller while the command
//! runs.
//!
//! [run]: Invocation::run
//! [start]: Invocation::start

pub mod argument;
pub mod program;
pub mod variable;
pub mod variable_name;
pub mod variable_value;
pub mod working_directory;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::process::Stdio;
use std::time::Instant;

use tokio::process::Command;

pub use self::argument::Argument;
pub use self::program::Program;
pub use self::variable::Variable;
pub use self::variable_name::VariableName;
pub use self::variable_value::VariableValue;
pub use self::working_directory::WorkingDirectory;
use crate::error::RunCommandError;
use crate::execution::Capture;
use crate::execution::Execution;
use crate::process_id::ProcessId;
use crate::run::Run;

/// The description of one external command
///
/// An invocation holds the program that the command starts, the arguments of
/// the program, and the directory in which the command runs. The arguments
/// keep the order in which the caller named them, and the working directory
/// is optional.
///
/// An invocation also holds the variables that the caller sets for the
/// command. The command inherits the environment of the process that runs
/// it, and a variable of the invocation replaces a variable of the process
/// with the same name.
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
///
/// A command that reads a variable of its environment:
///
/// ```
/// use kawauso_process::Invocation;
///
/// let invocation = Invocation::new("cargo")
///     .arg("nextest")
///     .arg("run")
///     .env("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1");
///
/// assert_eq!(
///     invocation.to_string(),
///     "NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run"
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

    /// The variables that the caller sets in the environment of the command
    ///
    /// The order is the order in which the caller set them, and a name
    /// appears once. The command inherits the environment of the process that
    /// runs it, and these variables come on top of that environment.
    environment: Vec<Variable>,

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
            environment: Vec::new(),
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

    /// Builds the command that a run starts
    ///
    /// Both forms of a run start the same command, so both take it from here.
    /// The command pipes both output streams, because a run reports what the
    /// command wrote to each of them.
    ///
    /// The command carries no call that clears the environment, so the
    /// command inherits the environment of this process. The variables of the
    /// invocation come on top of it, and a variable with the name of one that
    /// the process holds replaces it for the command alone. The program
    /// travels as the caller wrote it, and the operating system resolves a
    /// bare name.
    // process[impl run.environment]
    // process[impl run.resolution]
    fn command(&self) -> Command {
        let mut command = Command::new(self.program.get());

        command
            .args(self.arguments.iter().map(Argument::get))
            // process[impl invocation.environment]
            .envs(
                self.environment
                    .iter()
                    .map(|variable| (variable.name().get(), variable.value().get())),
            )
            // process[impl run.stdin]
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // process[impl run.abandonment]
            // process[impl stream.abandonment]
            .kill_on_drop(true);

        if let Some(directory) = &self.working_directory {
            command.current_dir(directory.get());
        }

        command
    }

    /// Sets one variable in the environment of the command
    ///
    /// The command inherits the environment of the process that runs it, and
    /// the variable comes on top of that environment. A variable with the
    /// name of one that the process holds replaces the value for the command
    /// alone, and the process keeps its own.
    ///
    /// A later call with the same name replaces the value of the earlier
    /// call, because a variable holds one value. The variable keeps the place
    /// of the earlier call.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("cargo")
    ///     .arg("build")
    ///     .env("RUSTUP_TOOLCHAIN", "nightly");
    ///
    /// assert_eq!(invocation.environment().len(), 1);
    /// ```
    // process[impl invocation.environment]
    pub fn env(mut self, name: impl Into<VariableName>, value: impl Into<VariableValue>) -> Self {
        self.set(Variable::new(name, value));

        self
    }

    /// Returns the variables that the caller set, in the order of the calls
    pub fn environment(&self) -> &[Variable] {
        &self.environment
    }

    /// Sets every variable of an iterator in the environment of the command
    ///
    /// Each variable goes through the same steps as one that [`env`][env]
    /// sets, so a later variable with the name of an earlier one replaces its
    /// value. A pair of a name and a value is a variable, so a caller that
    /// holds its variables as pairs gives them as they are. Use this method
    /// for a caller that holds its variables in a collection, and
    /// [`env`][env] for one that names them.
    ///
    /// # Examples
    ///
    /// ```
    /// use kawauso_process::Invocation;
    ///
    /// let invocation = Invocation::new("cargo")
    ///     .arg("build")
    ///     .envs([("RUSTUP_TOOLCHAIN", "nightly"), ("CARGO_TERM_COLOR", "always")]);
    ///
    /// assert_eq!(invocation.environment().len(), 2);
    /// ```
    ///
    /// [env]: Invocation::env
    // process[impl invocation.environment]
    pub fn envs(mut self, variables: impl IntoIterator<Item = impl Into<Variable>>) -> Self {
        for variable in variables {
            self.set(variable.into());
        }

        self
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

    /// Runs the command and reports what it produced
    ///
    /// The method starts the program, reads what the command writes to its
    /// standard output and to its standard error, and waits for the command
    /// to end. A command that fills a pipe stops until a reader empties it,
    /// so the run reads both streams while it waits. A command that writes
    /// more than a pipe holds therefore never stops the run.
    ///
    /// The command starts with a standard input that is null, so a command
    /// that reads its input sees the end of the input at once. A program that
    /// asks for a password ends instead of waiting for an answer. The command
    /// inherits the environment of the process that runs it, and it reads the
    /// variables of the invocation on top of that environment. The operating
    /// system resolves the program with the rules of the platform, and the
    /// crate searches no path of its own.
    ///
    /// A command that ends without success is no failure of this method. The
    /// status travels in the result, and [`require_success`][require-success]
    /// turns a status that is not a success into an error.
    ///
    /// The future of this method owns the command. A caller that drops the
    /// future, after a timeout for example, ends the command with it.
    ///
    /// On Windows, a program whose name ends in `.cmd` or `.bat` starts
    /// through `cmd.exe`. Rust escapes the arguments for that interpreter and
    /// refuses an argument that it cannot pass safely, so an argument reaches
    /// the program as the caller wrote it or the run fails.
    ///
    /// # Errors
    ///
    /// Returns [`UnstartableCommand`][unstartable] when the program does not
    /// start, and [`IncompleteRun`][incomplete] when the command started and
    /// the crate could not collect its result.
    ///
    /// # Panics
    ///
    /// Panics when no Tokio runtime drives the future. The runtime watches
    /// the streams and the end of the command, and the crate has no way to
    /// run the command without one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let execution = Invocation::new("git").arg("status").run().await?;
    ///
    /// println!("{}", execution.stdout());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [incomplete]: RunCommandError::IncompleteRun
    /// [require-success]: Execution::require_success
    /// [unstartable]: RunCommandError::UnstartableCommand
    // process[impl run]
    pub async fn run(&self) -> Result<Execution, RunCommandError> {
        let start = Instant::now();

        let child =
            self.command()
                .spawn()
                .map_err(|source| RunCommandError::UnstartableCommand {
                    invocation: self.clone(),
                    source,
                })?;

        // The wait takes the command, and the identifier of a command that
        // ended is gone. The run therefore reads the identifier here, so
        // that the result can carry it.
        // process[impl run.identity]
        let id = child.id().map(ProcessId::from);

        // The wait reads both streams while it waits for the end of the
        // command. A wait that read one stream after the other, or that read
        // nothing until the end, would stop on a command that fills a pipe.
        // process[impl run.drain]
        let output =
            child
                .wait_with_output()
                .await
                .map_err(|source| RunCommandError::IncompleteRun {
                    invocation: self.clone(),
                    source,
                })?;

        Ok(Execution::new(
            self.clone(),
            id,
            output.status,
            Capture::new(output.stdout),
            Capture::new(output.stderr),
            start.elapsed(),
        ))
    }

    /// Puts one variable into the environment of the command
    ///
    /// A variable with a name that the environment holds replaces the value
    /// of that variable and keeps its place, because a variable holds one
    /// value. A variable with a new name goes to the end, so that the
    /// environment keeps the order of the calls.
    // process[impl invocation.environment.replacement]
    fn set(&mut self, variable: Variable) {
        let existing = self
            .environment
            .iter_mut()
            .find(|existing| existing.name() == variable.name());

        match existing {
            Some(existing) => *existing = variable,
            None => self.environment.push(variable),
        }
    }

    /// Starts the command and reports its output while the command runs
    ///
    /// The method starts the program and returns a handle. The handle gives
    /// the output of the command to the caller as lines, in the order in
    /// which the lines arrive, and it collects the same capture that
    /// [`run`][run] collects. Use this method for an application that shows
    /// the progress of a command, or that turns the output of a command into
    /// events of its own, and [`run`][run] for one that needs the result
    /// alone.
    ///
    /// The command starts with the same decisions that [`run`][run] makes.
    /// The standard input is null, the command inherits the environment of
    /// the process that runs it, and the operating system resolves the
    /// program. The variables of the invocation reach the command as well.
    ///
    /// The handle owns the command. A caller that drops the handle kills the
    /// command with it, and a caller that asks the command to end first calls
    /// [`stop`][stop] on the handle.
    ///
    /// # Errors
    ///
    /// Returns [`UnstartableCommand`][unstartable] when the program does not
    /// start.
    ///
    /// # Panics
    ///
    /// Panics when no Tokio runtime runs. The runtime watches the streams and
    /// the end of the command, and the crate has no way to run the command
    /// without one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kawauso_process::Invocation;
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut run = Invocation::new("cargo").arg("build").start()?;
    ///
    /// while let Some(line) = run.next_line().await? {
    ///     println!("[{}] {line}", line.stream());
    /// }
    ///
    /// println!("{}", run.wait().await?.status());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [run]: Invocation::run
    /// [stop]: Run::stop
    /// [unstartable]: RunCommandError::UnstartableCommand
    // process[impl stream]
    pub fn start(&self) -> Result<Run, RunCommandError> {
        let started = Instant::now();

        let child =
            self.command()
                .spawn()
                .map_err(|source| RunCommandError::UnstartableCommand {
                    invocation: self.clone(),
                    source,
                })?;

        Ok(Run::new(self.clone(), child, started))
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
/// The line names every variable that the caller set, then the program, and
/// then every argument, each in the order in which the caller named them. A
/// variable shows as its name, an equals sign, and its value, which is the
/// form in which a shell sets a variable for one command. The line does not
/// name the working directory, because a command line holds the command and
/// not the place that it runs in.
///
/// The line is for a person. No caller reads it back, and no shell runs it,
/// so it is not a command line that a shell has to accept.
// process[impl invocation.display]
// process[impl invocation.environment.display]
impl Display for Invocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        for variable in &self.environment {
            write!(formatter, "{}=", variable.name())?;
            write_word(formatter, &variable.value().get().to_string_lossy())?;
            formatter.write_str(" ")?;
        }

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
/// The value of a variable is a word of this kind as well. The marks go around
/// the value alone, and not around the name, which is how a shell writes a
/// variable whose value holds a space.
///
/// The marks are for the reader. A word that holds a quotation mark of its own
/// keeps it, because no shell parses the line.
///
/// # Errors
///
/// Returns the error of the formatter when it cannot take the text.
fn write_word(formatter: &mut Formatter<'_>, word: &str) -> FmtResult {
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
