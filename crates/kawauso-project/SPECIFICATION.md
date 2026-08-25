# Projects for Kawauso

`kawauso-project` finds the project that a tool runs in. Every tool finds its
project in the same way, anchors its relative paths at the same directory,
and reads its configuration file from the same conventional location.
[ADR-009] records why the crate exists and how it relates to
`kawauso-config`.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Discovery

A tool that belongs to a project needs the directory of that project. It
anchors the relative paths of its configuration there, and it writes the
files that it generates there. The user runs the tool from the project or
from a directory in it, so the crate starts at a directory and walks up, one
directory at a time, until it finds the project.

A project announces itself with a marker: an entry that exists at a relative
path inside the directory, such as `.config/whisker.toml`, `.git`, or
`src/main.rs`. The developer names the markers of their tool, and the
configuration file of the application is always one of them. A marker is
only tested for its existence. A tool that has to check the contents of the
entry, such as a `main.rs` that holds a specific macro, checks the entry that
the search found.

The first directory in which any marker matches is the project. The walk
stays the outer loop: every marker is tested in one directory before any
marker is tested in the directory above it. A project therefore wins over
the directory that contains it, and a marker such as `.git` ends the walk at
the repository, so that an entry above the repository never takes part. The
crate reports the directory in which the marker matched, as the walk found
it. A tool never derives the directory from the path of a file, because that
derivation repeats the convention of the search and breaks the day that the
convention changes.

A marker that is absolute, or that leaves the directory of the walk, is a
mistake in the application. Such a value moves the search to a place that
the walk never reaches, so the crate reports it. A value that names no entry
at all is a mistake as well.

A walk that reaches the root of the file system without a match has found
no project. The error is the right default: a tool that generates files
must not write them into a directory that nothing marks as a project. Some
tools run outside any project as well, with a default configuration, and
for them the developer opts in to a fallback: the start of the walk is the
project, and it has no marker.

project[discover.walk]
The crate MUST search the start directory and each of its ancestors, up to
the root of the file system.

project[discover.order]
The crate MUST search the start directory before its ancestors, and each
ancestor before the ancestor above it.

project[discover.markers]
In each directory of the walk, the crate MUST test each marker, and a marker
matches when an entry exists at its relative path inside the directory.

project[discover.markers.order]
The crate MUST test the configuration file of the application before the
markers that the developer names, and those in the order in which the
developer names them.

project[discover.markers.walk]
The crate MUST test every marker in a directory of the walk before any
marker in the ancestor above it.

project[discover.markers.error.outside]
The crate MUST return an error, and MUST NOT panic, when a marker is not a
relative path inside a directory of the walk.

project[discover.precedence]
The crate MUST end the search at the first directory in which a marker
matches, and MUST NOT search the ancestors above it.

project[discover.result]
The crate MUST report the directory of the walk in which the marker matched
as the project, together with the marker that matched.

project[discover.error.missing]
The crate MUST return an error, and MUST NOT panic, when no marker matches
in any directory of the walk.

project[discover.error.missing.message]
The message of the error MUST name the start directory and every marker, in
the order of the test.

project[discover.fallback]
When the developer opts in, the crate MUST report the start directory as the
project, without a marker, instead of an error when no marker matches.

### Start

The walk starts at the working directory of the process, because that is
where the user runs the tool. A tool that takes a path, such as a linter
that checks one file, wants the project that governs that path instead, and
the working directory can be anywhere. The developer therefore names the
start of the walk when the tool has one.

A start that the developer names can be relative, and it can hold `.` and
`..` components. The walk goes up from the start one component at a time,
and a component that is `..` would take it through a directory that the
caller never named. The crate therefore makes the start absolute and removes
these components before the walk. It does not resolve symbolic links, so the
walk sees the tree that the caller named, and the paths that it reports are
paths that the caller recognizes.

The path that a tool takes often names a file. The project that governs a
file is the project of the directory that holds it, so the walk starts
there. A start that does not exist is most likely a mistake in the argument
of the user, and the error that no project exists would hide it. The crate
reports the start instead.

project[discover.start.working-directory]
The crate MUST start the walk at the working directory of the process when
the developer names no start.

project[discover.start.caller]
The crate MUST start the walk at the directory that the developer names.

project[discover.start.absolute]
The crate MUST resolve a relative start against the working directory of
the process, and MUST remove the `.` and `..` components of the start,
before the walk.

project[discover.start.file]
The crate MUST start the walk at the directory that holds the start when the
start is not a directory.

project[discover.start.error.unreadable]
The crate MUST return an error, and MUST NOT panic, when the start does not
exist or cannot be read.

project[discover.start.error.unknown-directory]
The crate MUST return an error, and MUST NOT panic, when it needs the
working directory of the process and cannot determine it.

## Configuration

Our projects keep the configuration files of their tools in the
subdirectory `.config`, one file per tool, with the name of the tool and the
extension `.toml`. The crate states this convention once, so that a tool
does not restate it and a user finds the file of every tool in the same
place. A tool whose host dictates another location, such as a GitHub Action
that reads `.github`, names that location instead.

The configuration file of an application is a marker of the search, and the
crate tests it before every other marker. The project therefore knows
whether the walk found the file, and it does not look at the file system a
second time to answer that.

A tool asks the project for its configuration and gets a value of the type
that the tool defines. The project derives the path of the file, and the
reading and the deserialization are the work of `kawauso-config`, so a
failure in the file is reported in the same words as for every other
configuration file. An absent file is a state that the tool decides on. A
tool whose configuration is required gets an error that names the path at
which the file has to go. A tool that runs without a configuration file asks
for the default of its type instead. The default replaces an absent file
only: a file that exists and cannot be loaded is a mistake that the default
must not hide.

project[configuration.location]
The configuration file of an application MUST be the file
`<application>.toml` in the subdirectory `.config` of the project.

project[configuration.location.custom]
The crate MUST use the relative path that the developer names as the
configuration file instead, when the developer names one.

project[configuration.marker]
The configuration file of the application MUST be a marker of the search.

project[configuration.load]
The crate MUST deserialize the configuration file of the project into a type
that the developer defines.

project[configuration.error]
The crate MUST return an error, and MUST NOT panic, when the configuration
file of the project cannot be loaded.

project[configuration.error.missing]
The crate MUST return an error whose message names the path of the
configuration file when the project has no configuration file.

project[configuration.default]
The crate MUST return the default value of the type instead of an error when
the project has no configuration file and the developer asks for the
default.

project[configuration.default.error]
The default MUST NOT replace an error of a configuration file that exists.

[adr-009]: ../../adrs/009-project-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
