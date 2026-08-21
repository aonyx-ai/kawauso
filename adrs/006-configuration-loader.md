# ADR-006: Configuration Loader

## Status

Accepted

## Context

`kawauso-config` deserializes contents that the caller has already read,
through the free function `from_str`. The crate now reads the
configuration file itself, and more capabilities will follow: discovery
through the XDG base directories, a walk up the parent directories of
the working directory, and the layering of sources that [ADR-004]
leaves open.

Each of these capabilities needs an entry point, and the entry point
is the part of a public API that is hardest to change: every
application calls it, and every future capability must fit into it. We
must choose its shape before the second capability ships, because entry
points that ship separately can only be unified through a breaking
change. The
candidates are a free function per capability, a method on the caller's
configuration type, a builder with a strategy setter, and a loader with
one constructor per source.

## Decision

`kawauso-config` has one entry point: the `Loader`.

1. **Every configuration comes from a loader.** A caller constructs a
   loader and calls its load method. The free function `from_str`
   retires: contents that the caller has already read construct a
   loader like every other source. A future way to find a configuration
   file arrives as a new constructor on the same type, and the load
   method stays the only operation that returns a configuration.

2. **A loader is complete by construction.** Each source of a
   configuration has its own constructor, and the constructor takes
   everything that its source needs. A loader without a source cannot
   exist, so the load method cannot fail with an "incomplete loader"
   error that no caller can act on. The compiler rejects the call that
   a runtime check would catch. The rule does not limit construction to
   one call: a strategy with many options can use a builder whose
   intermediate steps are their own types, as the typed-builder crate
   generates them, so that the compiler still checks completeness.

3. **The loader is one runtime type.** Every constructor returns the
   same type, and no type parameter encodes the source. An application
   selects its source at runtime — the path from a flag when the user
   gives one, a discovery strategy when the user does not — and both
   arms of that branch produce the same loader.

4. **The caller's type appears at the load method.** The loader is not
   generic over the configuration type; the load method is. Type
   inference lets the annotation on the result name the type, and one
   loader can load many times and into more than one type.

5. **The vocabulary of sources stays private.** The crate does not
   expose an enum of strategies while every strategy is a constructor
   with one obvious shape. A public enum is fixed with its first
   release. A private one can still change when the second and third
   strategies show which variants the crate needs.

## Alternatives

We considered these alternatives and rejected them for the reasons
below.

### A Builder With a Strategy Setter

A builder in the classic shape, such as
`Loader::new().strategy(Strategy::Path(path)).load()`, was the starting
point of this design. The builder has a half-built state:
`Loader::new().load()` names no source. The crate must then either
return a runtime error that no caller can act on, or encode the
missing source in a type parameter of the loader. The type parameter
breaks the runtime selection of a source, because the arms of a match
then produce different types. One constructor per source keeps the
flexibility of the builder and removes the half-built state.

### A Method on the Configuration Type

`Configuration::loader().load()` reads well, but a trait or a macro
must supply the method. A blanket implementation over every
deserializable type adds a loader method to every type that serde can
deserialize, including every field type of a configuration. A derive
macro avoids that pollution, but costs a proc-macro crate and its
compile time. Both alternatives add machinery, and the call they enable
is not clearer than `Loader::path(path).load()`.

### A Free Function per Source

`from_str` today, `from_file` tomorrow, `from_xdg` after that. Free
functions leave no room for options: every option that a source gains
becomes a parameter of every call or a new function. The surface grows
with the product of sources and options, while the loader grows with
their sum: one constructor per source, one method per option.

## Consequences

- A new source of configuration is an additive change: a constructor on
  the loader, with no breaking change for existing callers.
- The removal of `from_str` breaks its callers. The crate has no
  published release, so no application outside this repository is
  affected.
- Every source returns the same error type from the load method. A
  variant that one source can never produce is still visible to the
  callers of that source, and only the documentation says which sources
  produce which variants.
- The layering of multiple sources stays open, as [ADR-004]
  records. A layered configuration combines sources, and a loader with
  one source per construction does not express that yet. When layering
  arrives, it can compose loaders or add an accumulating constructor;
  that choice belongs to its own ADR, and the single entry point of
  this ADR is where it must fit.

[adr-004]: 004-configuration-crate.md
