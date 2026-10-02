# ADR-012: Project Data Directory

## Status

Accepted

## Context

An application that runs in a project sometimes writes files that must not
live in the repository. [Aurel], which releases Rust crates, will write a
`tracing` log to a file of its own for each run of its CLI. The log cannot go
into the repository: the `check-git` step of aurel requires a clean working
tree, so a log file in the repository fails the run that wrote it unless the
repository ignores it, and aurel builds release branches in temporary
worktrees that disappear with the log.

`kawauso-project` from [ADR-009] reports the root of a project and the path
of its configuration file, and the second point of that ADR says that the
conventions for the other things that a project holds arrive in the same
place. Both paths that the crate reports are inside the working tree. The
crate gives no place outside it, so each application that needs one chooses
a directory, a layout, and a name for the project on its own, and two
applications of the toolkit choose differently.

The choice is hard to reverse. The files that an application writes stay on
the disk of its users. When a later release moves the directory, renames a
segment of its path, or derives another name for the same project, the files
that users already have stay at the old path, and the application no longer
finds them. Nothing deletes them, and nothing tells the user that they exist.

The platforms disagree on where such files go. The [XDG Base Directory
Specification][xdg] separates data, state, and cache, macOS keeps all three
in `Library/Application Support`, and Windows separates application data that
roams between the machines of a user from application data that stays on one
machine. The name of the project is a second question: a project is a
directory that a marker identifies, and the same project exists in many
directories, as clones, as worktrees, and as checkouts that a user renamed or
moved.

We must decide which directory of the platform is the base, how the path
below it is laid out, where the name of the project comes from, and what the
crate does when it cannot give a directory.

## Decision

A project gives an application a data directory of its own, outside the
repository, at `<data>/<application>/projects/<identifier>/`.

1. **The base is the local data directory of the platform.** On Linux, and
   on the other systems that follow the XDG Base Directory Specification,
   the base is the directory that `XDG_DATA_HOME` names, or `.local/share`
   in the home directory of the user when the variable is not set, is
   empty, or does not hold an absolute path. On macOS, the base is
   `Library/Application Support` in the home directory of the user, and
   `XDG_DATA_HOME` has no effect there. On
   Windows, the base is the known folder for the local application data of
   the user, which `%LOCALAPPDATA%` names by default. These are rules of
   the base directory, not of the project, and a later decision about
   another directory of the user, such as the configuration directory of
   an application, can state its own base in the same shape.

2. **The path is `<data>/<application>/projects/<identifier>/`.**
   `<application>` is the name of the application that the builder of the
   project already takes for the configuration file. The segment `projects`
   keeps `<data>/<application>/` free for files that belong to the
   application and to no project. On macOS the configuration directory and
   the data directory are the same directory, so `config.toml` of the user
   strategy of `kawauso-config` sits beside `projects`, and no identifier
   can collide with it.

3. **The developer supplies the identifier.** The crate does not derive the
   identifier of a project from the project. The application knows what
   makes two checkouts the same project, for example a value in the
   configuration file that its users commit, and it keeps the identifier
   stable across clones, worktrees, and renames. The identifier is a type
   of its own that holds exactly one normal component of a path on every
   platform: it is not empty, it holds none of `/`, `\`, and `:`, it is
   not `.` or `..`, and it is not absolute. The colon is a rule of its own,
   because on Windows a value such as `C:` is a drive prefix that replaces
   the path that it joins, although it is not absolute. The rules are the
   same on every platform, because the identifier often comes from a file
   that users of all platforms share. The type rejects any other value with
   an error when the application creates it, so a malformed value fails
   before it reaches the project. Every way to create the identifier checks
   the rules, deserialization included, and the type has no constructor
   that cannot fail. The crate does not change the value to
   make it fit, because two values that a change makes equal share one
   directory without a sign of it. The rules also keep the path below
   `<data>/<application>/projects/`: an identifier can come from a file in
   a repository that the user cloned and did not write, and `..`, an
   absolute path, or a drive prefix would otherwise point the application
   at a directory anywhere on the disk.

4. **The application passes the identifier to the call.** A method on the
   loaded project, such as `Project::data_directory`, takes the identifier
   as an argument. The application often reads the identifier from the
   configuration file that the project loads, so it has the value only
   after the load, and the builder of `Project` does not take it.

5. **The directory is created on access.** The method creates the
   directory and its parents when they do not exist, and returns the path.
   Each call does this, so a directory that a user deleted while the
   application ran exists again on the next call. The load of a project
   still writes nothing, and a failure to create the directory cannot fail
   the load. The project does not check the identifier during the load
   either. An application that deserializes the identifier directly from
   its configuration file makes a malformed value fail the load, and an
   application whose files are optional reads the value as text and
   creates the identifier after the load.

6. **The call returns an error and has no fallback.** The method returns a
   `Result` with one error enum for the action, in the shape that [ADR-005]
   gives it. One variant states that the crate cannot determine the local data
   directory, which happens on a platform without one and on a system where the
   home directory of the user is unknown. Another variant reads "failed to
   create", carries the path, and has the cause of the I/O failure as its
   `source`. The crate does not fall back to another directory. An application
   whose files are optional, such as the log of aurel, reports the error as a
   warning and continues.

The specification of `kawauso-project` defines the requirements, and names
the types. This ADR records the layout on disk and the contract of the
identifier, because a change to either orphans the files of users.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### A State or Cache Directory

A log is state in the terms of XDG, and files that an application can
recreate are cache. But most applications need a data directory at some
time, and a log is only one use of the directory. `dirs::state_dir()`
returns no directory on macOS and Windows, so a state directory needs a
fallback to the data directory on two of three platforms anyway. A cache
directory tells the user and the system that they can delete its contents,
which is false for a file that the application expects to find again. Data
is also the easier concept for a user who looks for the files.

### The Roaming Data Directory

Windows separates the roaming application data, which follows a user to
each machine of a domain, from the local application data, which stays on
one machine. The files of a project, such as logs, belong to the checkout on
one machine and have no meaning on another. Roaming them costs time at each
sign-in and copies files about checkouts that do not exist on the other
machine. The two directories differ only on Windows.

### An Identity That the Crate Derives

The crate can derive the identifier from the project, so that the developer
names nothing. Each source of the value fails:

- A hash of the root is opaque, and a user who looks for the files cannot
  tell which directory belongs to which project.
- A sanitized path of the root is readable, but a deep root gives a name
  longer than some file systems accept, and two roots can sanitize to the
  same name.
- The name of the root together with a hash of its path is readable and
  unique, but it shares the next problem.
- Each value that comes from the root changes when a user renames or moves
  the checkout, which orphans the files, and it gives each worktree and each
  clone a directory of its own, although they are the same project.
- An identity from git, such as the URL of a remote or the first commit, is
  stable across clones. But a project is a directory that any marker
  identifies, and a project without git, or a repository without a remote,
  has no such identity.

### The Identifier on the Builder

The identifier can be an option of the builder of `Project`, with a type
state that makes the call compile only on a project that has an
identifier. But the builder needs the value before the load, and the load
is what reads the configuration file that often holds the value. An
application then loads its project twice, or reads the file with
`Loader::path` and bypasses the project, which [ADR-009] exists to avoid.
A malformed identifier that the builder rejects would also fail the load,
so an application whose files are optional could not open its project
because of a value that only the data directory uses.

### A Function Without a Project

The data directory depends only on the name of the application and on the
identifier, so a function can return it without a loaded project. An
application could then write a log for a run that failed to find or load
its project. But [ADR-009] makes the project the place for the conventions
of the things that a project holds, and the identifier names a project. A
function beside the project would give the toolkit a second place for one
convention, and an application that has no project has nothing for the
identifier to name.

### An Option Instead of a Result

The method can return an `Option` that is empty when the platform has no
data directory, and leave the creation to the application. But the creation
is I/O that can fail for reasons that a caller reports, such as a denied
permission, and an `Option` cannot carry the cause. It also gives the
application the step that the crate exists to own: an application that
forgets to create the directory fails at its first write, with an error
that names its own file and not the directory.

### The XDG Base on macOS

Many command-line tools keep their data in `.local/share` on macOS as well,
and read `XDG_DATA_HOME` there. But the user strategy of `kawauso-config`
follows the convention of Apple on macOS and ignores `XDG_CONFIG_HOME`, and
the data directory follows the same convention, so that the files of one
application sit in one place on that platform.

### A Fallback Directory

The crate can return a directory in the temporary directory of the system,
or in the root of the project, when it cannot give the data directory. The
root is the place that the feature avoids: a file there makes the working
tree dirty, which is the failure that aurel has today. A temporary directory
hides where the files went, and the system can delete it at any time, so the
application loses the files that it expected to find again. An application
that wants a fallback can choose one with the error in hand.

### Creation When the Project Loads

The builder can create the directory when it loads the project, so that the
method only returns a path. But a load then writes to the disk of the user,
and an application that never writes a file still creates a directory for
each project that it opens. A failure to create the directory also fails the
load, so an application whose files are optional cannot open its project.

## Consequences

- Every application of the toolkit keeps the files of a project in the same
  place, with the same layout, and a user who finds the directory of one
  application knows where to look for the next.
- The layout and the rules of the identifier are a contract with the users
  of each application. A change to the base, to the segments of the path,
  or to the rules of the identifier that this ADR states needs a new ADR
  that supersedes this one, and a migration of the files that users already
  have.
- The developer has to choose an identifier and keep it stable. A project
  that changes its identifier gets a new, empty directory, and the files at
  the old path stay there. The crate cannot detect this.
- The project does not remember the identifier. Each call takes it again,
  and an application that passes two identifiers to one project gets two
  directories. The application keeps the value consistent, for example by
  reading it from one place.
- Two checkouts with the same identifier share one directory, by design. An
  application whose checkouts write at the same time, such as two worktrees
  that run at once, has to name its files so that they do not collide.
- The crate gains a dependency on a library that knows the directories of
  the platforms, such as `dirs`, which `kawauso-config` already uses. No type
  of it appears in the public API, as [ADR-005] asks.
- An application that runs where the crate cannot determine the local data
  directory gets an error and no directory. The application decides whether that
  is a warning or a failure.
- The crate creates the directory and never deletes anything in it. Pruning
  and rotation of the files stay with the application.
- Additional rules of the identifier that make it portable, such as the
  reserved names of Windows, are requirements of the specification and not
  decisions of this ADR. The specification settles them before the first
  release with the data directory, because a rule that rejects an
  identifier that an earlier release accepted orphans the files of that
  project. Among them are names that end in a space or a dot, which Windows
  shortens, so that a name of only spaces names `projects` itself.
- Two identifiers that differ only in case, such as `Foo` and `foo`, share
  one directory on the default file systems of macOS and Windows, and two
  directories on Linux. An application that wants one directory on every
  platform uses identifiers that differ in more than case. The file
  systems merge other names as well: Windows removes trailing dots and
  spaces from a name, so `foo.` and `foo` are one directory, and macOS
  treats the composed and decomposed Unicode forms of a name as one name.
  The crate itself never maps two identifiers to one directory, and the
  portable rules that the specification adds can reject such values.
- The name of the application becomes a segment of the path, as it already
  is in the user strategy of `kawauso-config`. This ADR gives the name no
  rules, and the specification decides whether it gains the rules of the
  identifier.
- The XDG Base Directory Specification asks that a missing directory is
  created with the mode `0700`. The mode of the directories that the crate
  creates on Unix is a requirement of the specification and not a decision
  of this ADR.
- A user of Linux who looks for a log in the state directory of XDG, which
  is where XDG puts logs, does not find it there.
- The configuration directory of the user, which the user strategy of
  `kawauso-config` gives an application and which [ADR-009] lists as an
  open need, is not answered here. It exists without a project, and its
  decision can reuse the shape of the rules of the base directory from this
  ADR. On macOS that directory is `<data>/<application>/`, so the segment
  `projects` is reserved there for this ADR, and the decision about the
  configuration directory cannot use it for other files.
- An application gets a data directory only from a loaded project. A run
  that fails to find or to load its project has no data directory, and an
  application that wants a log of such a run writes it elsewhere.

[adr-005]: 005-error-handling-in-libraries.md
[adr-009]: 009-project-crate.md
[aurel]: https://github.com/aonyx-ai/aurel
[xdg]: https://specifications.freedesktop.org/basedir/latest/
