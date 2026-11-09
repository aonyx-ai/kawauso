# ADR-013: Remove Specifications

## Status

Accepted

## Context

[ADR-002] gives each crate a specification in `crates/<crate>/SPECIFICATION.md`
and uses [Tracey] to link each requirement to the code that implements it and
to the test that verifies it. The decision rests on a hypothesis: agents with
good specifications work more autonomously and need less human oversight.

In practice, the specifications have not guided our work for some time. The
behavior of a crate is also stated in two places: the requirements of the
specification and the documentation of the published crate. For example, the
specification of `kawauso-project` requires that the crate creates each data
directory with the mode `0700`, and the documentation of the data directory
already states this for the consumer of the crate. Each change to the
behavior must update both texts, and only the documentation reaches the users
of the crate.

The tool also has a cost. Tracey 2.0 is a pre-release, its releases lag
behind its main branch, and we build it from source at a pinned Git
revision. The project receives little maintenance. A build takes about twenty
minutes, and its build script needs node and pnpm to bundle a web dashboard.
`pnpm` is pinned in `mise.toml` for this build only. To keep CI fast, a
`Cache` workflow keeps a warm build, a composite action restores it, and every
CI job that does not run `check-specs` must disable Tracey explicitly, because
otherwise mise builds it.

We must decide whether we keep the specifications without Tracey, replace
Tracey, or remove the specifications.

## Decision

We remove the specifications, the requirement identifiers, and Tracey.

1. **The documentation states what a crate does.** The rustdoc of a crate
   documents its behavior, what it requires of the caller, and how it
   fails, as AGENTS.md already asks. The tests verify the behavior. There is
   no separate list of requirements.

2. **No requirement is lost in the removal.** Before we delete the
   specification of a crate, we compare each requirement with the
   documentation and the tests of the crate. A requirement that the
   documentation does not state moves into it, and a requirement without a
   test gets one.

3. **The references in the code go away.** The `[impl …]` and `[verify …]`
   comments refer to identifiers that no longer exist, so we delete them.

4. **Tracey leaves the toolchain.** The `check-specs` action, its pin in
   `mise.toml`, its configuration in `.config/tracey/`, and the CI steps
   that build, cache, and disable it go away.

5. **Earlier ADRs read "specification" as the documentation and the tests.**
   [ADR-003], [ADR-004], [ADR-005], [ADR-009], [ADR-010], [ADR-011], and
   [ADR-012] are accepted and do not change. Where they leave a detail to the
   specification of a crate, the documentation and the tests of the crate
   now decide it.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### Keep the Specifications Without Tracey

The specifications can stay as plain documents, with or without the
references in the code. But without a check, nothing reports a requirement
that the code no longer implements or a reference to an identifier that no
longer exists. This is the drift that the alternative "Documentation Only" of
ADR-002 describes, and the specifications would still repeat the
documentation.

### Replace Tracey With an Own Check

A small Rakko action can parse the identifiers and the references and report
broken ones. It removes the build of Tracey, but we then maintain a tool for
a practice that we do not use.

### Install a Release of Tracey

A release of Tracey installs from a prebuilt binary and needs no build. But
the releases lag behind the main branch that our configuration follows, and
the project receives little maintenance, so the dependency stays a risk.

## Consequences

- CI no longer builds or caches Tracey, the `Cache` workflow and the
  `restore-tracey` action go away, and the jobs no longer need to disable it.
  A new machine installs the tools of the repository without a twenty-minute
  build, and `pnpm` leaves `mise.toml`.
- The behavior of a crate has one description, its documentation, and the
  consumer of the published crate reads the same text that the authors
  maintain.
- No tool reports which behavior has a test. Reviews must check that a change
  to the behavior comes with documentation and a test.
- A change to the behavior no longer starts with a change to a requirement,
  and no version of a requirement marks the code that a change affects. The
  author and the reviewer find that code themselves.
- An agent that implements a crate reads the ADRs, the issue, and the
  documentation of the crate, not a list of requirements. AGENTS.md no
  longer defines a completed task by the coverage of a specification.
- A new crate no longer needs a placeholder requirement, a Tracey entry, or a
  specification.
- If we later want requirements that a tool checks, a new ADR decides the
  format and the tool from the beginning.

[adr-002]: 002-specifications.md
[adr-003]: 003-lints.md
[adr-004]: 004-configuration-crate.md
[adr-005]: 005-error-handling-in-libraries.md
[adr-009]: 009-project-crate.md
[adr-010]: 010-dot-config-convention.md
[adr-011]: 011-process-crate.md
[adr-012]: 012-project-data-directory.md
[tracey]: https://tracey.bearcove.eu/
