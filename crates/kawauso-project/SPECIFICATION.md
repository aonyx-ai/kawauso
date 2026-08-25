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

## Placeholder

The crate has no capability yet. Tracey needs a requirement in this document,
so the crate starts with a placeholder. The first capability of the crate
replaces it.

project[placeholder.add]
The crate MUST provide a function that returns the sum of two unsigned 64-bit
integers.

[adr-009]: ../../adrs/009-project-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
