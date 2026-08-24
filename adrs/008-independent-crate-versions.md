# ADR-008: Independent Crate Versions

## Status

Accepted

## Context

Kawauso is a set of crates and a facade that re-exports them ([ADR-007]).
Each crate is useful on its own ([ADR-004]), and the facade gives a developer
the whole toolkit through one dependency. The first release of
`kawauso-config` and `kawauso` is imminent, and it fixes how we version and
release the crates: the version numbers, the tags, and the changelogs become
part of the public record with that release.

Our other projects, such as Clawless, release every crate of a workspace in
lockstep. The workspace declares one version, every crate inherits it, and a
release bumps that version, publishes every crate, and documents every change
in one changelog and one GitHub release. Lockstep is simple: one number names
the state of the whole project, and one release covers every crate.

Kawauso will not be consumed that way for a long time. We build the framework
step by step, and our applications will take individual crates long before
they take the facade. A crate matures at its own pace: `kawauso-config` can go
through several releases while the facade does not change, and a new crate is
unfinished while older crates are stable. Under lockstep, every release
republishes crates that did not change, a crate cannot ship a fix without a
release of every other crate, and the version of a young crate advances with
its siblings and misrepresents its maturity.

Lockstep also assumes that the workspace is ready to release as a whole at the
time of every release. We develop complex features across many pull requests,
and the pull requests for different crates interleave on the main branch.
Every commit builds and passes all checks, but a crate in the middle of such a
feature is not ready for a release, while another crate may need one. A scheme
that can only release everything at once forces us to release half a feature
or to delay a fix.

## Decision

Each crate of the toolkit has its own version, its own changelog, and its own
releases. The workspace declares no version, and a release is a release of one
crate.

1. **Each crate declares its own version.** The manifest of a crate holds its
   version, and the workspace manifest holds none. A new crate starts at
   version 0.1.0 when it is ready for its first release, and the version of a
   crate changes only when the crate changes. Before 1.0, a new minor version
   is a breaking release and a new patch version is not, because that is how
   [Cargo resolves version requirements][cargo-semver].

2. **Each crate has its own changelog.** The changelog lives at
   `crates/<crate>/CHANGELOG.md`, follows [Keep a Changelog], and ships in the
   published package. It documents only the changes of its crate. A change
   that touches several crates gets an entry in the changelog of each crate
   that it touches.

3. **Each release is a release of one crate.** A release publishes one crate to
   crates.io, and it gets a tag and a GitHub release of its own. The tag names
   the crate and the version as `<crate>@<version>`, for example
   `kawauso-config@0.1.0`, because a version alone no longer identifies a
   state of the repository. This is the syntax with which Cargo itself names a
   package at a version. Several crates can be released from the same commit,
   and each of them gets its own tag and its own release.

4. **Crates are released selectively.** A release of one crate does not
   require a release of another crate. The crates that are not part of a
   release stay as they are, and their pending changes wait for a release of
   their own. The only exception is a dependency between crates.

5. **A crate requires a compatible range of a sibling.** A crate that depends
   on another crate of the toolkit requires the lowest version that compiles,
   never an exact version, and the workspace manifest declares that
   requirement once for every crate that has it. The requirement is raised in
   the change that needs a newer version. A breaking release of a crate leaves
   the range that its dependents require; every dependent must then raise its
   requirement, and a dependent that re-exports the types that changed must
   make a breaking release of its own. A non-breaking release requires nothing
   of the dependents, because their requirement already admits it.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### Lockstep Versions

The workspace declares one version, and every release publishes every crate
under it. This is the convention of our other projects, and it keeps one
number that names the state of the toolkit. But it republishes crates that
did not change, it produces releases with empty changelog entries, it cannot
hold one crate back while another one ships, and the version of a young crate
says nothing about its maturity. The one benefit, a number that names a
coherent state of the toolkit, the facade provides on its own: its
requirements name a set of crates that work together, and a developer who
wants the whole toolkit takes the facade.

### Independent Versions With One Changelog and One Release

Each crate has its own version, but one changelog and one GitHub release
describe every release of the workspace. A consumer of one crate must then
read the notes of crates that it does not use, and the notes of one crate
scatter across releases that belong to other crates. A GitHub release also
belongs to a tag, and a tag names one crate; a release that covers several
crates needs a tag that names none of them.

### Independent Versions Without Selective Releases

Each crate has its own version, but a release publishes every crate that has
pending changes. Most release tools that support independent versions work
this way. A release is then blocked by the least finished crate, which is the
situation that lockstep creates, and which our interleaved development makes
common rather than rare.

### Only the Facade Is Published

The facade is the only published crate, with one version, and the other crates
are internal to it. This contradicts [ADR-004], which states that each crate is
useful on its own, and [ADR-007], which requires that a library can depend on
the single crate. It also contradicts how our own applications will consume
the toolkit first.

## Consequences

- A version names one crate, not the toolkit. The facade takes the role of a
  toolkit version: its requirements name a set of crates that work together,
  and its changelog records when a crate joined the set or changed
  incompatibly.
- The facade releases rarely: when a crate makes a breaking release, when a
  new crate joins the toolkit, and when we raise a requirement on purpose. A
  non-breaking release of a crate reaches the consumers of the facade through
  the compatible range, without a release of the facade.
- A breaking release of a crate cascades into a release of every crate that
  re-exports its types. The cost grows with the number of dependents, and the
  releases must be published in dependency order, because the verification of
  a dependent resolves its dependency from the registry.
- Cargo rejects a requirement that the local crate does not satisfy, so a
  dependent cannot miss a breaking release of a sibling: the build fails until
  the requirement is raised. A requirement that is too low after a
  non-breaking release is not caught by the build, because inside the
  workspace the dependency always resolves to the local crate. The consumers
  of the published crate find such a requirement, not we. The shared
  requirement in the workspace manifest lowers this risk: it is the highest
  requirement of any crate in the workspace, so a crate that did not raise it
  is covered by a crate that did.
- A change must be attributed to the crates that it touches, so that each
  changelog gets its entries and a selective release knows what it contains.
  The release tooling of our other projects releases the whole workspace and
  cannot do this. We decide on the tooling in a separate step, and until then
  we make releases by hand.
- The releases page on GitHub interleaves the releases of all crates, and its
  marker for the latest release means nothing across crates. The tag names
  keep the page readable.
- Every crate has one more file to maintain, and a change that touches several
  crates edits several changelogs.
- The decision diverges from our other projects. A contributor who knows those
  has to learn this scheme, and the ADR is where they learn it.
- The decision is reversible in one direction only. A later release can align
  the versions of all crates, but the tags and the changelogs of earlier
  releases stay per crate.

[adr-004]: 004-configuration-crate.md
[adr-007]: 007-facade-crate.md
[cargo-semver]: https://doc.rust-lang.org/cargo/reference/semver.html
[keep a changelog]: https://keepachangelog.com/en/1.1.0/
