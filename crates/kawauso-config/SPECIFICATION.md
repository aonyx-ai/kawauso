# Configuration for Kawauso

`kawauso-config` loads the configuration files of our tools. Every tool finds,
loads, and deserializes its configuration file in the same way and reports
failures with the same clear errors. [ADR-004] records why the crate exists.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
word MUST has the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Loading

The caller supplies the path to a configuration file and a type that defines
the expected structure of the file. The crate reads the file, parses it as
TOML, and deserializes it into the type.

config[load.deserialize]
The crate MUST deserialize the contents of a TOML file into a caller-defined
type.

[adr-004]: ../../adrs/004-configuration-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
