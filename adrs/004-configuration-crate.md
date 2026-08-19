# ADR-004: Configuration Crate

## Status

Accepted

## Context

Kawauso is a set of crates that applications can use on their own or together.
The first crate must serve a need that all of our tools share. Configuration
is such a need. Almost every tool reads a configuration file with a deeply
nested structure. Each tool must find the file, parse it, turn it into a typed
Rust struct, and report a clear error when one of these steps fails.

Our own projects — [Labelflair][labelflair], [Aonyx Cloud][aonyx-cloud], and
[Scout][aonyx-scout] — serve as prior art, and they teach three lessons.
Every project writes the same code around a parsing library: code to find the
configuration file, to load it, and to report errors. Copies of that code
diverge: two projects wrote almost the same configuration crate, and the two
have already drifted apart. And a library easily leaks into a public API: two
projects re-export the error type of their library, so every caller couples
to it. A shared crate can codify our conventions and avoid these issues.

Configuration is also a good first crate for a second reason. The domain is
small and well understood, so the crate is a low-risk test of our process:
ADRs for decisions, a specification with [Tracey][tracey] references, and the
lints from [ADR-003][adr-003].

The open question is how much we build ourselves. Libraries such as
[config][config] and [figment][figment] exist, are mature, and offer a lot of
functionality. We must decide whether we build our own crate on top of the
basic building blocks, wrap one of these libraries, or use one of them
directly in each application.

## Decision

We build `kawauso-config` as the first crate of the toolkit.

1. **The crate encodes our conventions.** Its purpose is that every tool
   finds, loads, and deserializes its configuration file in the same way and
   reports failures with the same clear errors. The value of the crate is this
   uniformity, not novel functionality.

2. **We build on serde and toml directly.** Deserialization comes from
   [serde][serde], and parsing comes from [toml][toml]. These building blocks
   do the hard work, so the layer that we add is small. We do not add a
   configuration library as a dependency until we have a need that these
   building blocks cannot cover.

3. **The public API never exposes a backend.** Consumers of `kawauso-config`
   see our types and our errors only. If a future need, for example the
   layering of multiple sources, makes a library such as figment the right
   engine, we can adopt it inside the crate without a breaking change. This
   rule keeps today's decision cheap to reverse.

The specification of the crate defines its requirements. This ADR records why
the crate exists and why it does not stand on a configuration library.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### Use Figment or Config Directly in Each Application

Each application can add [figment][figment] or [config][config] to its own
dependencies. This is what we did in past projects, and it is the reason for
this ADR. The libraries parse and deserialize, but every project still writes
the same custom code around them: the search locations, the error messages,
and the glue between them. The conventions live in many places, drift apart,
and no shared crate can improve them for all tools at once. Every application
also couples its code to the API of a pre-1.0 library, and a breaking release
touches every tool instead of one crate.

### Wrap Figment

`kawauso-config` can be a thin wrapper around [figment][figment]. Figment is
well designed and tracks the provenance of every value. But its core model
merges many sources into an untyped map before it deserializes. Our current
need is one file into one struct, which uses none of this machinery. A wrapper
must also choose between two bad options: it re-exports figment types and
couples our public API to a pre-1.0 library, or it hides them and therefore
writes the same API layer that we write without the wrapper. When the layering
of multiple sources becomes a real need, the hidden backend rule in the
decision lets us revisit this choice without a breaking change.

### Wrap Config

The same reasoning applies to the [config][config] crate, which follows the
same model of merged, untyped sources. We know the crate from past projects.
Its error reporting is weaker than figment's, and it gave us no help with the
part that we repeated: the conventions around it.

## Consequences

- Every tool reads its configuration in the same way and reports the same
  errors, and an improvement to the crate reaches all tools at once.
- We own and maintain code that overlaps with mature libraries. The overlap is
  small, because serde and toml carry the hard parts, but it is not zero.
- The crate must not leak backend types through its public API. This is a
  constraint on every future change to the crate.
- This ADR does not settle how we support the layering of multiple sources.
  Our server projects already merge a bundled default with environment
  variables, so we expect this need soon. When `kawauso-config` takes on that
  use case, we decide between our own implementation and a library as an
  internal engine, and we record that decision in a new ADR.
- The crate is the first full pass through our process, and it will show
  whether ADRs, specifications, and lints work together as intended.

[adr-003]: 003-lints.md
[aonyx-cloud]: https://github.com/aonyx-ai/cloud
[aonyx-scout]: https://github.com/aonyx-ai/scout
[config]: https://crates.io/crates/config
[figment]: https://crates.io/crates/figment
[labelflair]: https://github.com/jdno/labelflair
[serde]: https://crates.io/crates/serde
[toml]: https://crates.io/crates/toml
[tracey]: https://tracey.bearcove.eu/
