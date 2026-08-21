# ADR-006: Configuration Loader

## Status

Accepted

## Context

`kawauso-config` deserializes contents that the caller has already read,
through the free function `from_str`. The crate now takes on reading the
file itself, and more capabilities are in sight: discovery through the
XDG base directories, a walk up the parent directories of the working
directory, and the layering of sources that [ADR-004] leaves
open.

Every one of these capabilities needs an entry point, and the entry
point is the part of a public API that is hardest to change: every
application calls it, and every future capability must fit into it. We
must choose its shape before the second capability arrives, because two
entry points that grow independently do not converge into one. The
candidates are a free function per capability, a method on the caller's
configuration type, a builder with a strategy setter, and a loader with
one constructor per source.

## Decision

`kawauso-config` has one entry point: the `Loader`.

1. **Every configuration comes out of a loader.** A caller constructs a
   loader and calls its load method. The free function `from_str`
   retires: contents that the caller has already read construct a
   loader like every other source. A future way to find a configuration
   file arrives as a new constructor on the same type, and the load
   method stays the one place where a configuration comes out.

2. **A loader is complete by construction.** Each source of a
   configuration has its own constructor, and the constructor takes
   everything that its source needs. A loader without a source cannot
   exist, so the load method cannot fail with an "incomplete loader"
   error that no caller can act on. The compiler rejects the call that
   a runtime check would catch.

3. **The loader is one runtime type.** Every constructor returns the
   same type, and no type parameter encodes the source. An application
   selects its source at runtime — the path from a flag when the user
   gives one, a discovery strategy when the user does not — and both
   arms of that branch produce the same loader.

4. **The caller's type appears at the load method.** The loader is not
   generic over the configuration type; the load method is. Type
   inference lets an annotation on the result name the type, and one
   loader can load more than once and into more than one type.

5. **The vocabulary of sources stays private.** The crate does not
   expose an enum of strategies while every strategy is a constructor
   with one obvious shape. A public vocabulary is fixed the day it
   ships; a private one can still change when the second and third
   strategies teach us what the variants really are.

## Alternatives

We considered these alternatives and rejected them for the reasons
below.

### A Builder With a Strategy Setter

A builder in the classic shape, such as
`Loader::new().strategy(Strategy::Path(path)).load()`, was the starting
point of this design. The builder has a half-built state:
`Loader::new().load()` names no source. The crate must then either
return a runtime error that no caller can act on, or encode the state
in a type parameter. The type parameter breaks the runtime selection of
a source, because the arms of a match produce different types. One
constructor per source keeps the extensibility of the builder and
removes the half-built state.

### A Method on the Configuration Type

`Configuration::loader().load()` reads well, but the method must come
from somewhere. A blanket implementation over every deserializable type
hangs a loader method on every type in scope that serde can
deserialize, including types that are not configurations. A derive
macro avoids that pollution and costs a proc-macro crate and its
compile time. Both are heavy machinery for a call that
`Loader::path(path).load()` expresses with the same clarity.

### A Free Function per Source

`from_str` today, `from_file` tomorrow, `from_xdg` after that. Free
functions leave no room for options: every knob that a source grows
becomes an argument of every call or a new function. N sources with M
options multiply into a surface that a loader with one constructor per
source keeps flat.

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
