# Processes for Kawauso

`kawauso-process` runs the external programs of a Kawauso application. Every
application starts a program in the same way. The caller names the program and
its arguments. The crate collects the output of the program, and it reports how
the program ended. [ADR-011] records why the crate exists and where its
boundary is.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Placeholder

The crate has no capability yet. Tracey needs a requirement in this document,
so the crate starts with a placeholder. The first capability of the crate
replaces it.

process[placeholder.add]
The crate MUST provide a function that returns the sum of two unsigned 64-bit
integers.

[adr-011]: ../../adrs/011-process-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
