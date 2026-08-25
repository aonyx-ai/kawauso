# ADR-009: Project Crate

## Status

Accepted

## Context

The tools that we build with Kawauso run inside a project, and most of them
need to know where that project is. [Whisker] anchors its ignore patterns and
the paths of its custom lint crates at the directory of the project.
[Clawless] generates a command into the crate that the user works in, so it
has to find that crate. Each of them walks up from a directory until it finds
what identifies the project, and each of them wrote that walk on its own.

`kawauso-config` from [ADR-004] finds a configuration file and loads it, and
its load method returns the deserialized value and nothing else. The walk
that finds the file knows which directory held it, but that knowledge does
not leave the crate. A tool that needs the directory has to derive it from
the path of the file, and the derivation repeats the search convention of the
crate: a file at `.config/whisker.toml` anchors beside `.config`, not inside
it, so the tool has to know how many components the convention added.
Clawless shows the failure. It accepts `src/main.rs` and a bare
`main.rs`, derives the directory with `.parent().parent()`, and for the bare
file that is the directory above the project.

The directory is not the only thing that the loader cannot give. The entry
that identifies a project is not always a configuration file: clawless looks
for `src/main.rs`, which has nothing to deserialize. A project does not
always have a configuration file: whisker treats a project without one as a
supported target, with empty patterns. And the walk of a tool that belongs to
a repository ends at the repository: whisker stops at the directory that
holds `.git`, so that a configuration file above the repository is never
read. In each case the project is the primary result, and the configuration
file is one thing that a project has, and can lack.

Where a project keeps the configuration file of a tool is a convention. This
repository keeps `.config/nextest.toml` and `.config/tracey/config.styx`, and
whisker moves its file to `.config/whisker.toml`. The convention is a
property of our projects, and today every tool restates it.

We must decide where this capability lives: in `kawauso-config`, as a new
strategy of the loader or as a module beside it, or in a crate of its own. If
it is a crate, we must also decide how it relates to `kawauso-config`,
because a project has a configuration, and the loader reads configurations.

## Decision

We build `kawauso-project` as a crate of its own. It finds the project that a
tool runs in, and it encodes the conventions for the layout of our projects.

1. **A project is a directory that a marker identifies.** A search starts at a
   directory and walks up, and it ends at the first directory that holds one
   of the markers that the developer names, such as `.config/whisker.toml`,
   `.git`, or `src/main.rs`. A marker is a relative path, and the search only
   tests whether an entry exists at it. The result is the directory as the
   search found it, together with the marker that matched. No tool derives
   the directory from the path of a file.

2. **The crate encodes the layout of a project.** Where a project keeps the
   configuration file of a tool is a convention of our projects, and the
   crate states it once: a tool asks the project for the path of its
   configuration file and gets the conventional location. A tool whose host
   dictates another location, such as [Labelflair], a GitHub Action that
   reads `.github`, names that location instead. Conventions for the other
   things that a project holds arrive in the same place.

3. **The project loads its configuration through kawauso-config.**
   `kawauso-project` depends on `kawauso-config`, and never the reverse. A
   method on the project, such as `Project::configuration`, derives the path
   from the convention, decides what an absent file means, and delegates the
   read and the deserialization to `Loader::path`. The loader stays the only
   code that reads and deserializes a configuration, which is the substance
   of [ADR-006]; the method on the project is a caller of the loader, not a
   second loader. The project adds what the loader cannot know: which file,
   and whether a project without it is a supported state. The search has
   already tested the marker in the directory in which it ended, so the
   project answers that without a second look at the file system.
   `Loader::path` stays public, and a tool that needs an option the project
   does not offer calls the loader itself.

4. **The ancestors strategy of kawauso-config stays as it is.** It serves a
   tool without a project: the nearest file wins, and the walk ends at the
   root of the file system. A condition that ends the walk and a start other
   than the working directory are properties of the project search, and the
   ancestors strategy does not gain them. Two walks with the same options
   would be one walk in two places.

The specification of the crate defines its requirements. This ADR records why
the crate exists and how it relates to `kawauso-config`.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### A Search Inside kawauso-config

The project search can be a module of `kawauso-config`, with a constructor
`Loader::project` that reads a file at a fixed path inside a project. The
ancestors strategy is already a walk with the configuration file as its only
marker, so the step is small. But the purpose of the crate is to load
configuration files, and a search that returns a directory does not load one:
clawless would depend on a configuration crate and never load a
configuration. `Loader::project` is `Loader::path` with the directory joined
onto the path, so it is a second constructor for a source that [ADR-006]
gives one constructor. And the conventions for the layout of a project have
no home in a crate about files.

### The Loader Reports the Directory

The load method can gain a sibling that returns the value together with the
directory in which the walk found the file. The change is small, and it is
additive under [ADR-006]. But it serves only a tool that has a configuration
file. A tool without one has to deserialize into a placeholder type, and the
unit type does not work as one, because a TOML document is always a table. A
project without a file has no directory of a walk to report. The condition
that ends the walk and the caller-supplied start would then arrive as options
of the ancestors strategy, where they burden a strategy that most tools use
without them.

### A Project Crate That Does Not Depend on kawauso-config

The crate can find the project and derive paths from the convention, and a
tool composes the two crates: it asks the project for the path and hands it
to `Loader::path`. Neither crate then depends on the other. But the tool has
to decide on its own what an absent file means, and every tool writes the
same check around the load, although the search already knows whether the
marker matched. A consumer of a project should not have to think about how
its configuration is loaded; that is the framework half of [ADR-007]. The
coupling costs a compatible range on the sibling, which [ADR-008] asks of
every crate that depends on another crate of the toolkit.

### Markers That Check Contents

A marker can be a closure, so that clawless moves its check for the
`clawless::main!` macro into the walk. But a closure turns the marker into
code: an error cannot name it, a specification cannot state it, and two
tools cannot compare theirs. The walk and a check on its result give clawless
the same behavior in two steps, so a marker stays an entry that exists.

## Consequences

- Every tool finds its project in the same way, anchors relative paths at the
  same directory, and reads its configuration from the same conventional
  location. An improvement to the search reaches every tool at once.
- The toolkit gains a crate, with a specification, a changelog, a README, a
  Tracey entry, a module `kawauso::project` in the facade, and releases of
  its own under [ADR-008].
- `kawauso-project` requires a compatible range of `kawauso-config`, as
  [ADR-008] asks, and raises it in the change that needs a newer version. No
  type of `kawauso-config` appears in its public API, which [ADR-005] asks
  of every dependency, so a breaking release of `kawauso-config` does not
  force one of `kawauso-project`.
- The framework has two routes to a configuration: the method on the project
  and `Loader::path`. They cannot disagree, because one calls the other, but
  the documentation of a tool has to show one, and ours show the project.
- A developer chooses between two walks: the ancestors strategy for a tool
  without a project, and the project search for a tool with one. The
  questions that are open on the ancestors strategy, where the walk ends and
  where it starts, are answered by the project search and stay closed on the
  strategy.
- What the search reports when no marker matches, and what the project
  returns for an absent configuration file, are requirements of the
  specification and not decisions of this ADR.
- A tool that writes files into the directory of a user, which the user
  strategy of `kawauso-config` gives an application, has a need of the same
  shape, and neither crate answers it yet.

[adr-004]: 004-configuration-crate.md
[adr-005]: 005-error-handling-in-libraries.md
[adr-006]: 006-configuration-loader.md
[adr-007]: 007-facade-crate.md
[adr-008]: 008-independent-crate-versions.md
[clawless]: https://github.com/aonyx-ai/clawless
[labelflair]: https://github.com/jdno/labelflair
[whisker]: https://github.com/aonyx-ai/whisker
