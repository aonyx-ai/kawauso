# Configuration for Kawauso

`kawauso-config` loads the configuration files of our tools. Every tool finds,
loads, and deserializes its configuration file in the same way and reports
failures with the same clear errors. [ADR-004] records why the crate exists.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Loading

The caller supplies the path to a configuration file and a type that defines
the expected structure of the file. The crate reads the file, parses it as
TOML, and deserializes it into the type. Each step can fail. An error from
the read names the path, and an error from the parse or the deserialization
names the place in the file that has to change.

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

[adr-004]: ../../adrs/004-configuration-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
