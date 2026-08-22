# ADR-007: Facade Crate

## Status

Accepted

## Context

Kawauso is a set of crates. `kawauso-config` from [ADR-004] is the first one,
and more will follow. The crate `kawauso` is still a placeholder. It carries
the name of the project, so it is the name that a developer searches for
first, and the first release puts whatever it holds under semantic
versioning. We must decide what the crate is before then.

The answer follows from what we want Kawauso to be. Each crate is useful on
its own, which [ADR-004] states, but that is the smaller half of the goal.
The larger half is that a developer gets one strongly opinionated framework
through one import: one way to load a configuration, one shape of error, and
one set of conventions that runs through every part. Both halves are true at
once, and the framework is the half that most developers reach for.

Today only the smaller half exists. An application that wants three
capabilities writes three dependencies, three version requirements, and three
names in its imports, and it must already know which crate holds which
capability before it can look for one. The crate that carries the name of the
project offers nothing.

## Decision

`kawauso` becomes the facade of the toolkit: the crate through which a
developer takes the framework as a whole. It re-exports the other crates and
has no content of its own.

1. **The facade re-exports, and does nothing else.** The crate declares no
   type and no function. Every item that `kawauso` exposes belongs to another
   crate of the toolkit. A convenience that lives only in the facade is a
   second place where the API of the toolkit lives, and two places drift
   apart. It also splits the specification of a capability across two crates.

2. **A crate becomes a module with its own name.** `kawauso-config` is
   `kawauso::config`, which is the name of the crate without the prefix
   `kawauso-`. The rule needs no list of exceptions, so a reader who knows the
   crate can guess the path, and a reader who knows the path can guess the
   crate.

3. **A module is the crate, not a copy of it.** The facade re-exports the
   crate itself, so `kawauso::config::Loader` and `kawauso_config::Loader` are
   one type. An application can take the facade, a library can take the single
   crate, and a value still passes between them. The facade therefore requires
   a compatible range of each crate and never an exact version, because an
   exact version cannot unify with the requirement of an application that also
   depends on the crate directly.

4. **The facade takes every crate, and has no features.** A dependency on
   `kawauso` is a dependency on the whole framework, because that is what a
   developer who types the name of the project asks for. An application that
   wants a part of it depends on the crates that hold that part, which is
   shorter to write than a list of features.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### No Facade

`kawauso` stays unpublished, and every application depends on each crate that
it uses. This is the smallest change: one line of `publish = false` retires the
placeholder, and nothing else moves. But Kawauso is then a family of crates and
never a framework. A developer who has heard of the project finds no crate to
start from, a developer who has found one crate learns nothing about the
others, and the name of the project belongs to nobody on crates.io.

### A Facade With an API of Its Own

The facade re-exports the crates and adds convenience on top, such as a
function that loads a configuration with the defaults that we prefer. The
convenience is real, but it makes the facade a crate with behavior, and
behavior needs a specification, tests, and its own errors. Worse, it gives
every capability two entry points that must agree. When they disagree, the
application that depends on the single crate gets the worse one, because the
better one is in a crate it does not use. Convenience of this kind belongs in
the crate that owns the capability, where every consumer reaches it.

### A Feature per Module

Each module sits behind a feature with the name of the module, and the default
features contain all of them. An application then compiles only the crates
that it names. But a crate of the toolkit can depend on another crate of the
toolkit, and its feature must then switch that module on as well, or the crate
is compiled and its module is unreachable. The facade therefore has to mirror
the dependency graph that the manifests already state. The copy is silent when
it is wrong: the build succeeds, and a module is missing. Only an application
that wants a strict subset gains anything, and that application writes one
dependency on the crate that it needs instead.

## Consequences

- One dependency, one version requirement, and one name give a developer the
  whole framework, and the documentation of `kawauso` lists what the framework
  contains.
- Every capability has two paths, one through the facade and one through its
  crate. Documentation and examples must choose one, or a reader meets two
  names for one type.
- A new crate in the toolkit grows the build of every application that takes
  the facade, and a crate with heavy dependencies grows it a lot. An
  application that cannot pay for that leaves the facade and depends on the
  crates that it needs.
- A breaking change in any crate is a breaking change in the facade, because
  the facade re-exports the types that changed. The facade can therefore never
  release a smaller major version than the crates behind it. When and how the
  crates release is a decision that this ADR does not make, and the facade
  gives that decision a constraint to satisfy.
- The decision stays open. Features that are on by default can arrive later
  without a breaking change for an application that takes the defaults, so a
  crate that makes the cost real can reopen this question.
- The placeholder disappears before the first release, so no application ever
  depends on it.

[adr-004]: 004-configuration-crate.md
