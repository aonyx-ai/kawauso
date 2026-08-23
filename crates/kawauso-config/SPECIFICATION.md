# Configuration for Kawauso

`kawauso-config` loads the configuration files of our tools. Every tool finds,
loads, and deserializes its configuration file in the same way and reports
failures with the same clear errors. [ADR-004] records why the crate exists.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Discovery

A tool can ask its user for the path of its configuration file, but a flag
that the user must type on every run is friction. The crate finds the file
instead. The developer supplies the name of their application, and the crate
derives the name of the file from it.

The crate has two strategies, and the developer selects one. An application
that belongs to a project searches the working directory and its ancestors.
An application that belongs to a user searches the configuration directory
that its platform defines. A strategy never falls back to the other one, so
the user always knows where the application looks. When the search finds no
file, the error names every location that the crate searched, so that the
user knows where to put the file. A directory with the name of
the configuration file is a mistake that is hard to see, so the crate reports
it instead of a file from another location.

config[discover.strategy]
The crate MUST search only the locations of the strategy that the developer
selects.

config[discover.load]
The crate MUST deserialize the file that it finds into a type that the
developer defines.

config[discover.error]
The crate MUST return an error, and MUST NOT panic, when the search does not
find a configuration file.

config[discover.error.missing]
The message of the error MUST name every location that the crate searched, in
the order of the search, when no location has a configuration file.

config[discover.error.directory]
The crate MUST end the search with an error, and MUST NOT panic, when a
searched location has a directory with the name of the configuration file.

config[discover.error.directory.path]
The message of the error MUST name the path of that directory.

### Ancestors

The configuration of a project belongs to the project, and a user runs the
application from the project or from a directory in it. The crate therefore
starts at the working directory and goes up, one directory at a time, until
it reaches the root of the file system. The file that is nearest to the
working directory wins, so a project can override the configuration of the
directory that contains it.

config[discover.ancestors.name]
The crate MUST search for a file that has the name of the application and the
extension `.toml`.

config[discover.ancestors.working-directory]
The crate MUST search the working directory of the process.

config[discover.ancestors.parents]
The crate MUST search each ancestor of the working directory, up to the root
of the file system.

config[discover.ancestors.order]
The crate MUST search the working directory before its ancestors, and each
ancestor before the ancestor above it.

config[discover.ancestors.precedence]
The crate MUST use the first file that it finds, and MUST NOT search the
locations that follow it.

config[discover.ancestors.error.unknown-directory]
The crate MUST return an error, and MUST NOT panic, when it cannot determine
the working directory of the process.

### User Directory

The configuration of a user belongs to the user, and every platform has a
place for it. The crate follows the [XDG Base Directory Specification] on
Linux, the convention of Apple on macOS, and the convention of Microsoft on
Windows, so that the file sits where the rest of the platform sits. Each of
these places has a requirement of its own, because a phrase such as "the
configuration directory of the operating system" leaves every reader with a
different path in mind. The application gets a directory of its own in that
place, which leaves room for the files that it writes later.

`XDG_CONFIG_HOME` belongs to the standard of one platform, and the crate
reads the variable only where that standard applies. A user of macOS who
sets it therefore keeps the file in `Library/Application Support`.

config[discover.user.xdg]
On Linux, and on the other systems that follow the [XDG Base Directory
Specification], the crate MUST search the directory that the environment
variable `XDG_CONFIG_HOME` names.

config[discover.user.xdg.default]
On these systems, the crate MUST search `.config` in the home directory of
the user when `XDG_CONFIG_HOME` is not set, is empty, or does not hold an
absolute path.

config[discover.user.macos]
On macOS, the crate MUST search `Library/Application Support` in the home
directory of the user.

config[discover.user.windows]
On Windows, the crate MUST search the directory for the roaming application
data of the user.

config[discover.user.name]
The crate MUST search for a file with the name `config.toml` in a directory
that has the name of the application.

config[discover.user.error.unknown-directory]
The crate MUST return an error, and MUST NOT panic, when it cannot determine
the configuration directory of the platform.

## Loading

The caller supplies the path to a configuration file and a type that defines
the expected structure of the file. The crate reads the file, parses it as
TOML, and deserializes it into the type. Each step can fail. An error from
the read names the path, and an error from the parse or the deserialization
names the place in the file that has to change. A directory at the path is
the same mistake that the search finds, so the crate reports it in the same
words.

config[load.deserialize]
The crate MUST deserialize the contents of a TOML file into a caller-defined
type.

config[load.error]
The crate MUST return an error, and MUST NOT panic, when the contents of a
configuration file cannot be deserialized into a caller-defined type.

config[load.error.syntax]
The message of the error MUST state the line and the column of the failure
when the contents are not a valid TOML document.

config[load.error.field]
The message of the error MUST name the path in the document where the failure
occurred, such as `server.port`, when a valid TOML document does not match the
caller-defined type.

config[load.file]
The crate MUST read the file at a caller-supplied path and deserialize its
contents into a caller-defined type.

config[load.file.error]
The crate MUST return an error, and MUST NOT panic, when it cannot read the
file at a caller-supplied path.

config[load.file.error.missing]
The message of the error MUST name the path when no file exists at that path.

config[load.file.error.unreadable]
The message of the error MUST name the path when a file exists at that path
but cannot be read.

config[load.file.error.directory]
The message of the error MUST name the path, and MUST state that it is a
directory, when a directory exists at that path.

[adr-004]: ../../adrs/004-configuration-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
[xdg base directory specification]: https://specifications.freedesktop.org/basedir/latest/
