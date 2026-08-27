# ADR-010: Dot-Config Convention

## Status

Accepted

## Context

`kawauso-project` keeps the configuration file of an application at
`.config/<application>.toml`, and the ancestors strategy of `kawauso-config`
reads `<application>.toml` in each directory of its walk and in the
subdirectories that the developer names. Both follow the [dot-config]
convention, which asks a tool to keep its configuration under `.config` in the
directory of a project.

The convention describes itself as an extension of the [XDG Base Directory
Specification][xdg] from the home directory of a user to the directory of a
project. XDG gives each application a directory of its own,
`$XDG_CONFIG_HOME/<application>`, and the user strategy of `kawauso-config`
reads `config.toml` in that directory. The extension to a project therefore
has two layouts: a file `.config/<application>.toml`, and a directory
`.config/<application>/` with `config.toml` inside. Neither dot-config nor XDG
chooses between them. The tools that dot-config lists keep the file, and this
repository keeps `.config/nextest.toml` beside `.config/tracey/config.styx`. A
tool that follows the convention has to accept both.

The two layouts differ in who owns the directory, and the difference explains
the two file names. `.config` is one directory for many tools, so a tool
qualifies its file with its own name. `.config/<application>` belongs to one
application, so `config.toml` is enough. `.github` is a directory of the first
kind, and the documentation of `Subdirectory` names it together with `.config`
as a convention, which hides that the second kind exists.

[Aonyx] needs the directory. It keeps its operations, its workflows, a
generated SDK, and an ontology schema in `.config/aonyx/`, and a user who
opens the directory expects to find the configuration file there. Aonyx reads
the file through the ancestors strategy as well as through the project. Today
it names `.config/aonyx/config.toml` itself, which repeats the convention and
the name of the application in the application.

The ancestors strategy cannot express the directory. It computes one file
name, `<application>.toml`, and joins it onto every location of the search. A
subdirectory names where the search reads, not what it reads, so the option
`.subdirectory(".config/aonyx")` reads `.config/aonyx/aonyx.toml`, a file that
no convention describes.

We must decide where the convention lives, what shape the option takes, and
which layout wins when a project has both.

## Decision

Both crates support both layouts of the dot-config convention.

1. **The ancestors strategy gains the convention as one option.**
   `AncestorsSearch::dot_config` adds `.config/<application>.toml` and
   `.config/<application>/config.toml` to every directory of the walk. The
   convention fixes the base directory `.config` and the file name
   `config.toml`, so the developer names neither. The rule is the one that
   the user strategy already follows: an application that owns a directory
   keeps `config.toml` in it.

2. **A location resolves its own paths.** The search holds one ordered list
   of locations, in the order in which the developer named them, and each
   location produces the paths that the search reads in a directory of the
   walk. A subdirectory is a location that produces
   `<subdirectory>/<application>.toml`, and the convention is a location that
   produces its two paths. One list keeps the order of the calls, so a
   subdirectory that the developer names before the convention still wins
   over it. The vocabulary of locations stays private, as the vocabulary of
   sources does under [ADR-006]: two methods on the search name the two
   kinds, and a public type waits until a third kind shows which shape the
   type needs. The getter that returns the subdirectories goes away, because
   it no longer describes the search.

3. **The file comes before the directory.** In each directory of the walk,
   the search reads `.config/<application>.toml` before
   `.config/<application>/config.toml`. An application that creates its
   directory for the other files that it owns must not change which
   configuration file its user has. The order also follows the rule of the
   list, where a directory comes before its subdirectories.

4. **The project selects the directory as an option.**
   `with_configuration_directory` on the builder of `Project` makes
   `.config/<application>/config.toml` the configuration file of the project.
   The option excludes a custom file and the declaration that the application
   has no configuration file, as these two exclude each other today. The
   conventional location stays `.config/<application>.toml`. The project does
   not search both layouts, because it reports the one path at which the file
   goes, and a tool that creates the file needs one answer.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### A Parameter That Names the Directory

The search can take the directory that the application owns as a parameter,
such as `.application_directory(".config")`, and the project can take the
same. But the convention fixes the directory, and a parameter invites
`.github`, which produces `.github/<application>/config.toml`, a path that no
tool reads and no convention describes. The developer would also repeat the
convention in every application, which is the repetition that the crates
exist to remove.

### A File Name for the Search

The search can take the name of the file as an option, such as
`.file_name("config.toml")`, next to the subdirectory `.config/<application>`.
But the option applies to every location of the search, so
`.config/<application>.toml` and `.config/<application>/config.toml` cannot
coexist in one search. The developer also repeats the name of the application
in the subdirectory, which is what the issue set out to remove.

### A Named Subdirectory

`Subdirectory` can gain a constructor for the convention, such as
`Subdirectory::dot_config()`, which keeps the getter and the public type. But
the value that the getter returns then names a directory, `.config`, that is
not what the search reads, and a subdirectory that resolves to two paths is no
subdirectory. The type would carry a kind that its name denies.

### A Strategy of Its Own

The convention can be a constructor on the loader, such as
`Loader::dot_config`. But the constructor is the ancestors walk with different
locations, and the reasoning of [ADR-009] applies: two walks with the same
options are one walk in two places. A tool that accepts `.github` as well as
the convention would also need both strategies in one search, which one
strategy with two kinds of location gives it.

### The Project Alone

The option can live in `kawauso-project` only, which is what the issue
proposes. But Aonyx reads its configuration through the ancestors strategy as
well, and the strategy would stay unable to express a layout that our own
projects use. The project also loads through `kawauso-config`, and a
convention that the loader cannot express is one that the two crates state
differently.

### The Project Searches Both Layouts

The project can read whichever of the two files exists, as the search does.
But the project reports the path of the configuration file so that a tool can
create the file, and a project with two locations has no one path to report.
A file at the other location would then be one that the project reads but
never writes, and "no configuration" would mean that neither of two files
exists.

### The Directory Before the File

The search can read `.config/<application>/config.toml` before
`.config/<application>.toml`. A user who moves the file into the directory and
forgets the old one then gets the new one. But an application that creates
its directory for other files then changes which configuration its user has,
without a change to either file, and the search would read a subdirectory
before the directory that contains it, against the rule of the list.

## Consequences

- Both crates state the convention once, and an application that follows it
  names itself once. Aonyx replaces a path that repeats its name with
  `with_configuration_directory`.
- The getter for the subdirectories leaves the public API of
  `kawauso-config`, which is a breaking change and a release 0.3.0 under
  [ADR-008]. No crate outside this repository calls the getter, and
  `kawauso-project` does not use it.
- `kawauso-project` gains the option in a minor release. It does not need the
  new `kawauso-config`, because it joins a path and hands it to
  `Loader::path`, so its compatible range does not move.
- The requirement that names the file of the ancestors strategy,
  `discover.ancestors.name`, changes its text and gets a new version, and the
  specification of each crate gains requirements for the convention. The
  documentation of `Subdirectory` separates the directory that many tools
  share from the directory that one application owns.
- The order costs the user something. A user who moves the file into the
  directory and keeps the old one does not see the move, because the old
  file wins. A warning about a file that the search passed over is a feature
  of its own, and this ADR does not decide it.
- The two crates read the convention differently, by design. The loader with
  `dot_config` reads both layouts, and the project reads one. A project whose
  developer selected the directory reports no configuration when its user
  kept the file.
- The vocabulary of locations stays private. A third kind of location, or a
  caller that has to inspect the list, decides whether it becomes public.
- The directory that the convention gives an application is a configuration
  directory. This ADR does not give a tool a home for the other files that it
  owns. Aonyx keeps its workflows and its SDK there today, and a home for
  such files is a question of its own.

[adr-006]: 006-configuration-loader.md
[adr-008]: 008-independent-crate-versions.md
[adr-009]: 009-project-crate.md
[aonyx]: https://github.com/aonyx-ai/aonyx
[dot-config]: https://dot-config.github.io/
[xdg]: https://specifications.freedesktop.org/basedir/latest/
