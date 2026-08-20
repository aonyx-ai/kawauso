# ADR-005: Error Handling in Libraries

## Status

Accepted

## Context

Kawauso is a toolkit of library crates, and the errors of a library are part
of its public API. The first crate, `kawauso-config` from [ADR-004][adr-004],
raises the question immediately: loading a configuration file can fail when
the file is read, parsed, or deserialized, and each failure comes from a
different library underneath. The crate must report these failures in a way
that the caller can act on, without exposing the libraries that produced them.

Part of the answer is code style: how an error type is named and structured
is settled by convention, and a convention can evolve with the codebase. The
boundary of a library cannot. What a caller can do with an error, how the
error relates to the dependencies underneath, and how the error type stays
stable while the crate evolves are commitments that every consumer builds
against. Those commitments are the subject of this ADR.

The Rust ecosystem has converged on a small set of answers. Mature libraries
such as `std::io`, hyper, reqwest, sqlx, and the AWS SDK never expose the
error types of their dependencies; causes travel type-erased behind the
library's own types, and hyper even documents the chain behind
`Error::source` as unstable. The literature — the [API guidelines][c-good-err],
[Sabrina Jewson][jewson], [matklad][matklad], and the
[AWS SDK's design RFC][rfc0022] — separates two purposes that one error must
serve: a small, coarse surface for the cases a caller branches on, and rich
context in `Display` for the human who reads the report. Blanket
`From<DependencyError>` impls are the recognized way a dependency leaks into
a public API by accident, and `#[non_exhaustive]` on public error enums is
the standard way to add variants without a breaking change.

## Decision

Error types in Kawauso crates follow these rules.

1. **One error type per fallible action.** Every operation that can fail
   returns its own error enum, named after the action and defined next to
   it. The signature of an operation then states exactly which failures it
   can produce, and no crate-wide enum grows into a union of unrelated
   operations.

2. **An error serves two audiences.** The variants of an error enum exist for
   the caller that branches: a variant is added when a caller can plausibly
   act on the distinction, and not before. Everything the caller only reads
   goes into context fields and the `Display` message: the path, the key, the
   position, and what was expected. Fine-grained variants for failures that
   every caller treats the same add API surface without a consumer.

3. **Dependency types never cross the boundary.** No type from a dependency
   appears in a public signature, a variant field, or a `From` impl. A cause
   from a dependency travels as a `Box<dyn Error + Send + Sync>` or inside an
   opaque newtype of ours with a private field. The chain behind
   `Error::source` exists for reporting and logging; its concrete types are
   not part of the API, and callers must not downcast them for control flow.
   Types from the standard library are exempt. This generalizes the hidden
   backend rule of [ADR-004][adr-004] to the error types of every crate.

4. **Each error describes its own layer.** The `Display` message of an error
   states what failed at its own level and never repeats the message of its
   source. Causality lives in the `source` chain, and a reporter that walks
   the chain prints each layer once. A message that embeds its source's text
   produces the duplicated "failed to load: failed to parse: failed to parse"
   output that this rule exists to prevent.

5. **Public error enums are `#[non_exhaustive]`, and so are their struct
   variants.** The attribute on the enum lets a crate add a variant without a
   breaking change, and the attribute on the variants lets a crate add
   context fields without one. Callers match with a wildcard arm and bind
   fields with `..`; this cost is small, because the caller that exhaustively
   branches on every variant does not exist in practice.

6. **Errors implement the standard traits.** Every public error type
   implements `std::error::Error`, `Send`, `Sync`, and `Debug`, so it
   composes with the reporting tools of the ecosystem. How a crate implements
   them — by hand or with a derive such as `thiserror` — is an implementation
   detail.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### One Error Enum per Crate

A single crate-wide enum that unions every failure is the most common pattern
in the ecosystem, and the literature is unanimous against it. The enum grows
into a catch-all that couples unrelated operations, no caller can know which
variants a given function actually returns, and every embedded dependency
type blocks the crate's own stability. The first rule of the decision scopes
error types to one per fallible action instead.

### Opaque Struct With a Kind

Libraries such as `std::io`, hyper, and reqwest hide the error behind an
opaque struct and offer a `kind()` enum or `is_*` predicates. The pattern
earns its complexity when the set of failure categories itself churns, which
is true for an HTTP client and not for our crates, whose failure spaces are
small and domain-defined. A `#[non_exhaustive]` enum gives our callers direct
matching with the same freedom to grow. A future crate whose failure space is
open-ended can adopt the opaque pattern in its own ADR.

### Type-Erased Errors in the Public API

Returning an `anyhow`-style erased error from a library is ergonomic for the
author and useless for the caller: the signature no longer says what can
fail, and the only way to branch is to downcast to concrete types the caller
learns about out of band. The ecosystem consensus reserves erased errors for
applications, and we follow it. Our tools remain free to use `anyhow` at the
binary level to report the structured errors our crates return.

### Exposing or Re-Exporting Dependency Errors

A crate can re-export the error type of its backend, or embed it in a public
variant field, and save itself the wrapping layer. This is the cheapest
option today and the most expensive one later: every caller couples to the
dependency, a semver-major release of the dependency breaks every tool, and
the backend can never be swapped. This is the leak that [ADR-004][adr-004]
forbids for `kawauso-config`, and this ADR forbids it for every crate.

### Diagnostic Frameworks in the Public API

Frameworks such as `miette` render compiler-quality reports with spans and
labels, and even miette's own documentation tells libraries not to put its
types in their public API, because the framework becomes a public,
semver-relevant dependency for every consumer. We keep rendering out of the mandatory
surface: a crate carries structured context and a good `Display`, and a
`Diagnostic` derive can arrive later behind an optional feature without a
breaking change.

## Consequences

- Every crate reports failures in the same shape, and a caller of one Kawauso
  crate knows how to handle the errors of all of them.
- A crate can swap a backend, as ADR-004 anticipates for `kawauso-config`,
  without a breaking change to its error types.
- Wrapping causes behind opaque newtypes and writing per-layer messages costs
  boilerplate that a blanket `From` impl would avoid. We accept this cost;
  the `From` impl is exactly the leak this ADR prevents.
- Callers must write wildcard arms and `..` patterns, and they lose the
  compiler's missing-variant diagnostics when a crate adds a variant.
- A caller that needs structured detail from a cause cannot downcast for it;
  the crate must expose an accessor. This is intentional: it forces the need
  into the public API, where it is designed instead of leaked.
- The specification of each crate still defines which failure cases exist and
  what context they carry; this ADR only constrains their shape.

[adr-004]: 004-configuration-crate.md
[c-good-err]: https://rust-lang.github.io/api-guidelines/interoperability.html
[jewson]: https://sabrinajewson.org/blog/errors
[matklad]: https://matklad.github.io/2020/10/15/study-of-std-io-error.html
[rfc0022]: https://smithy-lang.github.io/smithy-rs/design/rfcs/rfc0022_error_context_and_compatibility.html
