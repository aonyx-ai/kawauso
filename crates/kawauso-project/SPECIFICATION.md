# Projects for Kawauso

`kawauso-project` finds the project that a Kawauso application runs in. Every
application finds its project in the same way. It anchors its relative paths
at the directory of the project, and it reads its configuration file from the
same conventional location. [ADR-009] records why the crate exists and how it
relates to `kawauso-config`.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Discovery

An application that belongs to a project needs the directory of that project.
The application reads the resources of the project below this directory, and
it writes the files that it creates there. The crate starts at a directory
and walks up, one directory at a time, until it finds the project.

A marker identifies a project. A marker is an entry at a relative path in the
directory, such as `.git`, `src/main.rs`, or `.config/example.toml`. The
developer names the markers of their application, and a search has at least
one marker. The crate tests only whether the entry exists. Some applications
have to examine the contents of the entry, such as a `main.rs` file with a
specific macro. These applications examine the entry that the search found.

The first directory with a marker that matches is the project. The walk stays
the outer loop: the crate tests every marker in one directory before it tests
any marker in the directory above. A project therefore wins over the
directory that contains it. A marker such as `.git` ends the walk at the
repository. The crate then never reads an entry above the repository.

The crate reports the directory in which the marker matched, as the walk
found it. An application never derives this directory from the path of a
file. Such a calculation repeats the convention of the search, and it breaks
when the convention changes.

A marker that is absolute, or that leaves the directory of the walk, is a
mistake in the application. Such a value moves the search to a place that the
walk never reaches. A value that names no entry at all is a mistake as well.
The crate reports these values instead of a project.

A walk that reaches the root of the file system found no project. The crate
then returns an error. Without this error, an application that creates files
can write them into a directory that no marker identifies.

project[discover.walk]
The crate MUST search the start directory and each of its ancestors, up to
the root of the file system.

project[discover.order]
The crate MUST search the start directory before its ancestors, and each
ancestor before the ancestor above it.

project[discover.markers]
In each directory of the walk, the crate MUST test each marker. A marker
matches when an entry exists at its relative path in the directory.

project[discover.markers.required]
The crate MUST reject a search that names no marker.

project[discover.markers.order]
The crate MUST test the markers in the order in which the developer names
them.

project[discover.markers.walk]
The crate MUST test every marker in a directory of the walk before any marker
in the ancestor above it.

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
The crate MUST return an error, and MUST NOT panic, when no marker matches in
any directory of the walk.

project[discover.error.missing.message]
The message of the error MUST name the start directory and every marker, in
the order of the test.

### Start

The developer names the directory at which the walk starts. An application
that takes a path, such as a linter that examines one file, needs the project
that governs that path. The user gives this path as an argument.

A start can be relative, and it can contain `.` and `..` components. The walk
goes up from the start one component at a time. A `..` component moves the
walk through a directory that the caller never named. The crate therefore
resolves a relative start against the working directory of the process, and
it removes these components before the walk. The crate does not resolve
symbolic links. The walk therefore sees the tree that the caller named, and
it reports paths that the caller recognizes.

The path that an application takes often names a file. The project that
governs a file is the project of the directory that holds the file. The walk
therefore starts at that directory. A start that does not exist is usually a
mistake in the argument of the user. A report that no project exists hides
this mistake, so the crate reports the start instead.

project[discover.start.caller]
The crate MUST start the walk at the directory that the developer names.

project[discover.start.absolute]
The crate MUST resolve a relative start against the working directory of the
process. It MUST also remove the `.` and `..` components of the start before
the walk.

project[discover.start.file]
The crate MUST start the walk at the directory that holds the start when the
start is not a directory.

project[discover.start.error.unreadable]
The crate MUST return an error, and MUST NOT panic, when the start does not
exist or cannot be read.

project[discover.start.error.unknown-directory]
The crate MUST return an error, and MUST NOT panic, when it needs the working
directory of the process and cannot determine it.

[adr-009]: ../../adrs/009-project-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
