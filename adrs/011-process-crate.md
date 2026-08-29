# ADR-011: Process Crate

## Status

Accepted

## Context

Two applications of the toolkit need to run external programs and capture
what they produce. [Clawless] wants an interface for external commands that
integrates with its output system ([clawless#154]), and [Rakko] wraps every
maintenance tool as a subprocess ([rakko#27], [Rakko ADR-011]). The
mechanics are identical in both: start a program with an argument vector,
collect its output, and report how it ended.

The mechanics look small, and they are not. A child that writes more than
the capacity of a pipe blocks until the parent reads from it, so a parent
that waits for the exit before it reads, or reads the two streams one after
the other, deadlocks on a talkative child. A tool that npm installs on
Windows is a `.cmd` shim that only `cmd.exe` can start, with escaping rules
of its own. A run that a caller abandons must end the child, or the child
outlives the tool that started it. Each consumer that writes its own run
loop meets these traps alone, and a fix in one tool does not reach the
other.

What differs between the consumers is the integration, not the engine.
Clawless streams the output of a child into its event system as the output
arrives; Rakko resolves the program through mise before the run. Underneath
both sits the same engine: spawn, drain, wait, report.

The engine is concurrent by nature. It drains two streams while it waits
for the exit, and a caller may observe the output while the child runs.
Kawauso has no async crate today, so the capability brings a runtime
question with it. We must decide where the engine lives, and whether the
toolkit takes its first async dependency to build it.

## Decision

We build `kawauso-process` as a crate of its own. It runs one external
program, captures what the program produces, and reports how the program
ended. The crate is the engine that the consumers share; the integration
that distinguishes them stays with them.

1. **The crate is the engine, and integration stays with the consumers.**
   An invocation names the program, the arguments, and optionally the
   working directory, and a run yields the exit status, the captured bytes
   of both streams, and the elapsed duration. Clawless builds its wrapper
   that streams into events on top of this, and Rakko adds tool resolution
   through mise. Neither belongs in the crate, because an opinion of one
   framework in the engine burdens every other consumer.

2. **The API has two levels.** A one-call form runs the command and
   captures everything, which is all that Rakko needs. A handle form yields
   the output as lines in arrival order, tagged by stream and decoded
   lossily for display, while the same capture accumulates; it exists so
   that Clawless can turn the output of a child into events while the child
   runs. Waiting on the handle returns the same result as the one-call
   form, so a consumer that moves from one form to the other keeps its
   handling.

3. **The crate is async and builds on tokio.** Draining two streams while
   waiting for an exit is exactly what an async runtime expresses, and both
   consumers already run one: Clawless creates its runtime in the command
   runner, and the harness of Rakko drives its actions with one. Tokio is
   the first async dependency of the toolkit, and this ADR records that
   step deliberately. Per [ADR-005], no tokio type appears in a public
   signature: causes travel type-erased or as `std::io::Error`, which is
   exempt as a standard library type.

4. **A non-zero exit is data, and an abandoned run is a kill.** Consumers
   read exit codes as results — the check mode of a formatter fails with
   findings — so the result of a run carries the status, and the error type
   is reserved for the run itself failing, in the shape that [ADR-005]
   gives it. A convenience that requires success serves the caller that
   wants success-or-explain, with the command line, the status, and the
   captured stderr in its message. Dropping the handle, or the future of
   the one-call form, kills the child. That is the entire cancellation
   story, and it composes with timeouts, with select, and with a scheduler
   that abandons a run.

5. **No shell touches the command.** The program is a path or a bare name
   that the operating system resolves; the crate never searches `PATH`.
   Arguments pass as a vector, and nothing splits, quotes, or expands them.
   Stdin is null, so a child can never hang on input that no one sends. The
   child inherits the environment of the parent. Each restriction keeps a
   later extension additive: an API for the environment arrives when a
   consumer needs one, and the write side of the handle is where
   interactive input would land.

6. **Windows is a first-class target.** Rust 1.77.2 and later spawn `.cmd`
   and `.bat` files through `cmd.exe` with strict escaping and refuse an
   argument that cannot pass safely, so an argument arrives as written or
   the run fails loudly. The crate documents this, and the minimum
   supported Rust version of the workspace stays at or above 1.77.2. Line
   splitting handles `\r\n`, and the raw byte capture stays untouched.

The specification of the crate defines its requirements, and names and
signatures follow the conventions of this repository. This ADR records why
the crate exists, where its boundary lies, and that async enters the
toolkit with it.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### A Run Loop in Each Consumer

Each consumer writes its own spawn, drain, and wait, which is where both
issues start today. The mechanics are identical, so two loops encode one
design twice, each meets the traps of the context alone, and a fix for a
deadlock or an escaping rule reaches one tool. [ADR-009] closed the same
duplication for the walk that finds a project; this crate closes it for
the run.

### The Engine Inside Clawless

Clawless asked first, and its command interface could carry the engine as
a crate of the framework that Rakko then depends on. But a maintenance
tool then builds part of a CLI framework that it does not use, the engine
releases at the pace of the framework, and the boundary between engine and
integration sits inside one owner that has no reason to keep it. The
toolkit is the home for mechanics that every application shares, which is
why the crate lives in Kawauso and not in either consumer.

### A Synchronous Crate on std::process

A synchronous crate keeps the runtime out of the toolkit. But the engine
must drain two pipes while it waits, and a synchronous crate does that
with a thread per stream; the threads become behavior of the crate, with a
lifecycle and panics of their own. Cancellation loses its expression: a
blocked read has no portable interruption, so an abandoned run either
leaks the child or gains a kill method that every consumer must remember
to call. Both consumers already run a tokio runtime, so the synchronous
form serves a consumer that does not exist. When one appears, a blocking
convenience can wrap the async form; the reverse path does not exist.

### Direct Use of tokio::process

`tokio::process` spawns a child and waits for it, and each consumer can
call it directly, with no new crate. But the library stops where the traps
begin: the caller drains the pipes, tags and decodes the lines, renders
the command for a log, and attaches the captured stderr to a failure.
Every consumer writes that layer again, which is the first alternative
with more of the work left in it. The crate we build uses `tokio::process`
underneath; the decision is not whether the library is good, but where the
layer above it lives.

### Commands as Shell Strings

A single string that a shell splits is the form that a terminal teaches.
But `sh` and `cmd.exe` split and quote by different rules, an argument
with a space or a metacharacter changes its meaning silently, and the
shell expands characters that the caller meant literally. The argument
vector states the command exactly, on every platform. A consumer that
wants a shell names the shell as the program and passes the command line
as one argument.

## Consequences

- Every tool runs external programs in the same way, and the difficult
  parts — the concurrent drain, the Windows escaping, the kill on drop —
  exist once. An improvement to the engine reaches every tool at once.
- The toolkit gains a crate, with a specification, a changelog, a README,
  a Tracey entry, a module `kawauso::process` in the facade, and releases
  of its own under [ADR-008].
- Async enters the toolkit, and tokio becomes a dependency of the
  workspace. The facade takes every crate ([ADR-007]), so an application
  that depends on `kawauso` builds tokio even when it never runs a
  process. ADR-007 left the question of default features open for the
  crate that makes this cost real; this crate may be the one that reopens
  it.
- The crate requires a tokio runtime at call time. No signature states the
  requirement, a call without a runtime fails at run time, and the
  documentation must say it prominently.
- The kill that ends an abandoned child is immediate, and the child cannot
  clean up after itself. A consumer that needs an orderly shutdown needs
  an API that does not exist yet, and the handle is where it would arrive.
- A caller that must not accept a non-zero exit has to check the status,
  and a caller that forgets the check reads the output of a failed run as
  a result. The convenience that requires success makes the check one
  call.
- The Windows behavior ships as design and unit tests, because no consumer
  has Windows CI today. Until a Windows job exists, a regression on
  Windows reaches a user before it reaches us. The escaping guarantee also
  binds the toolchain: the minimum supported Rust version must stay at or
  above 1.77.2, which the current 1.85.0 of the workspace satisfies.
- What the result type carries, how the builder reads, and what each
  failure variant names are requirements of the specification and not
  decisions of this ADR.

[adr-005]: 005-error-handling-in-libraries.md
[adr-007]: 007-facade-crate.md
[adr-008]: 008-independent-crate-versions.md
[adr-009]: 009-project-crate.md
[clawless]: https://github.com/aonyx-ai/clawless
[clawless#154]: https://github.com/aonyx-ai/clawless/issues/154
[rakko]: https://github.com/aonyx-ai/rakko
[rakko#27]: https://github.com/aonyx-ai/rakko/issues/27
[rakko adr-011]: https://github.com/aonyx-ai/rakko/blob/main/adrs/011-tool-integration.md
