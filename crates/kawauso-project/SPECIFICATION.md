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

Some applications also run outside a project, with default settings. Their
developer can select the start directory as the project instead of the error.
This project has no marker, because no marker matched.

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

project[discover.fallback]
The crate MUST report the start directory as the project when the developer
selects this fallback and no marker matches. This project MUST have no
marker.

### Start

Most applications run inside the project that they work on. The user starts
them in the project, or in a directory below it, so the working directory is
already inside the project. An application that takes no path from its user
therefore starts the search there.

Some applications, linters for example, take a path as an argument. That path
can be outside the working directory, so the developer can also name another
directory as the start of the search.

A start can be relative, and it can contain `.` and `..` components. The walk
goes up from the start one component at a time. A `..` component moves the
walk through a directory that the caller never named. The crate therefore
resolves a relative start against the working directory of the process, and
it removes these components before the walk.

The crate also resolves the symbolic links of the start. A `..` component
after a symbolic link leaves the tree that the link points into. A walk that
keeps the link therefore passes through a directory that the user never
entered. The crate reports a canonical path for the project. An application
that joins a path onto that directory reaches the entry that the walk saw. An
application that shows the path that its user typed keeps that path itself.

The path that an application takes often names a file. The project that
governs a file is the project of the directory that holds the file. The walk
therefore starts at that directory. A start that does not exist is usually a
mistake in the argument of the user. A report that no project exists hides
this mistake, so the crate reports the start instead.

project[discover.start.working-directory]
The crate MUST start the walk at the working directory of the process when
the developer declares no explicit start.

project[discover.start.caller]
The crate MUST start the walk at the directory that the developer names.

project[discover.start.absolute+2]
The crate MUST resolve a relative start against the working directory of the
process. It MUST also remove the `.` and `..` components of the start, and
resolve its symbolic links, before the walk.

project[discover.start.file]
The crate MUST start the walk at the directory that holds the start when the
start is not a directory.

project[discover.start.error.unreadable]
The crate MUST return an error, and MUST NOT panic, when the start does not
exist or cannot be read.

project[discover.start.error.unknown-directory]
The crate MUST return an error, and MUST NOT panic, when it needs the working
directory of the process and cannot determine it.

## Configuration

Our projects keep the configuration file of a tool in the subdirectory
`.config`, with the name of the application and the extension `.toml`. The
crate states this convention once, so that an application does not repeat it
and a user finds the file of every tool in the same place. An application
whose host dictates another location, such as a GitHub Action that reads
`.github`, names that location instead.

The dot-config convention adds a second layout. An application that owns the
directory `.config/<name>` keeps the file `config.toml` in it, next to the
other files that it owns. A directory that belongs to one application does
not repeat the name of that application in the file, so the application names
itself once. The developer selects the directory with an option, and the
crate then reads the file `config.toml` in it and reports that one path. The
two layouts are two states, so a project never reads both.

The developer describes the project once, before the crate finds it: where the
search starts, which markers identify the project, and which file holds the
configuration. The crate then reads the file and deserializes it into a type
that the developer defines. A caller that holds the project can therefore ask
for the configuration and get a value, and no caller reads the file a second
time.

Not every project has a configuration file. A project without one is a normal
state, not a failure, so the crate reports no configuration and the
application decides what to do. An application whose configuration is
required states that itself, and an application that runs with default
settings asks its type for them.

A file that exists and cannot be read or deserialized is a different case. The
user wrote that file and expects the application to use it, so the crate
reports the failure instead of the absence.

Some applications have no configuration file at all. Such an application wants
only the directory of its project, and a file at the conventional location
belongs to something else. The developer therefore declares that the
application has no configuration file. The crate then reads no file, and it
reports no configuration. It still reports the conventional location, because
an application that writes the file later needs to know where the file goes.

project[configuration.location]
The configuration file of an application MUST be the file
`<application>.toml` in the subdirectory `.config` of the project.

project[configuration.location.custom]
The crate MUST use the relative path that the developer names as the
configuration file instead, when the developer names one.

project[configuration.location.directory]
The crate MUST use the file `config.toml` in the directory
`.config/<application>` of the project as the configuration file, when the
developer selects the configuration directory.

project[configuration.load]
The crate MUST deserialize the configuration file of the project into a type
that the developer defines.

project[configuration.missing]
The crate MUST report no configuration, and MUST NOT return an error, when
the project has no configuration file.

project[configuration.error]
The crate MUST return an error, and MUST NOT panic, when the configuration
file of the project exists and cannot be loaded.

project[configuration.none]
The crate MUST NOT read a configuration file, and MUST report no
configuration, when the developer declares that the application has no
configuration file.

### Creation

Some applications need a configuration file that no user wrote. Such an
application gives each project a value that it makes once, an identifier for
example. It keeps that value in the file of the project. A project without the
file therefore needs one before the application can do its work. Every such
application writes the same file at the same location, so the crate writes it
instead.

The developer asks for the creation, and the crate then writes the file when
the project has none. The developer supplies the value, because a value that
an application makes for one project has no constant that describes it. The
crate reports the value that it wrote as the configuration of the project. A
caller therefore reads a configuration for a new project and for an old one.

Creation is the only write. A file that a user wrote holds their comments and
the order of their fields, and a serializer that writes the whole document
loses both. The crate therefore does not repair a file that exists. A file
that the type of the application rejects fails the load, the error names the
problem, and the user corrects the file.

A read that writes surprises the caller, so a load alone creates nothing. A
project without a configuration file stays a normal state, and the developer
states the intent to create with a different method. An application that
declares that it has no configuration file creates none either. The two
statements contradict each other, and the crate rejects the pair.

The search runs before the write, and a search that finds no project fails.
The crate therefore does not write a file outside a project.

project[configuration.create]
The crate MUST create the configuration file of the project from a value that
the developer supplies, when the developer asks for the creation and the
project has no configuration file.

project[configuration.create.directories]
The crate MUST create the directories that the configuration file needs.

project[configuration.create.result]
The crate MUST report the value that it wrote as the configuration of the
project.

project[configuration.create.existing]
The crate MUST NOT write to the configuration file of the project when a file
exists at its location.

project[configuration.create.load]
The crate MUST NOT create a configuration file when the developer does not ask
for the creation.

project[configuration.create.none]
The crate MUST reject the creation of a configuration file for an application
that declares that it has no configuration file.

project[configuration.create.error]
The crate MUST return an error, and MUST NOT panic, when it cannot create the
configuration file of the project.

[adr-009]: ../../adrs/009-project-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
