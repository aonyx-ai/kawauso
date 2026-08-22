# Kawauso

`kawauso` is the toolkit as a whole. It re-exports the other crates of the
toolkit as modules, so that an application needs one dependency and one
version requirement to reach all of them. [ADR-007] records why the crate has
this shape.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the toolkit.

## Facade

The crate holds no code of its own, and every item that it exposes belongs to
another crate. A developer reaches a crate of the toolkit under a short name,
and the name follows from the name of the crate, so a reader who knows one can
derive the other.

An application can also depend on a crate of the toolkit directly, and a
library that it uses can depend on the facade. Both must work on the same
values. A module is therefore the crate itself, and never a copy of it.

The crate has no features, and a dependency on it brings every module. An
application that wants a part of the framework depends on the crates that hold
that part instead.

kawauso[facade.module]
The crate MUST provide every other crate of the toolkit as a module with the
name of that crate without the prefix `kawauso-`.

kawauso[facade.identity]
A module MUST be the crate that it provides, so that a type from the module
and the same type from the crate are one type.

[adr-007]: ../../adrs/007-facade-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
