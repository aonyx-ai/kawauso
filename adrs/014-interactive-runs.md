# ADR-014: Interactive Runs

## Status

Accepted

## Context

[ADR-011] gave `kawauso-process` a run that captures what a program writes.
Its stdin is null, and it pipes both output streams. [Clawless] also needs to
run programs that take over the terminal: `$EDITOR`, a pager, and `ssh`
([clawless#798]). Such a program reads the keys of the user and draws on the
terminal, so its streams must be the streams that the caller chooses, and
not pipes.

The first version of that feature copied the spawn and the stop of the crate
into Clawless. A review found three faults in the copy. The stop signaled the
program alone, so a wrapper such as `sh -c`, `code --wait`, or the pager of
Git left a grandchild that held the terminal. A Ctrl+C in the program also
reached Clawless, which then cancelled the command around the editor. The
kill on drop reached the program alone as well. ADR-011 says that consumers
must not grow a run loop of their own, so the start belongs in this crate.

The traps are those of job control on Unix. A terminal sends the signals of
Ctrl+C, Ctrl+\\, and Ctrl+Z to its foreground process group. A process that
reads the terminal from another group receives `SIGTTIN` and stops. A process
outside the foreground group that changes the foreground group receives
`SIGTTOU`, unless it ignores or blocks that signal. A signal reaches every
process of a group only if the caller sends it to the group.

## Decision

We add an interactive start to `kawauso-process`. It starts an invocation
with the streams that the caller chooses, and it returns a handle that waits
for the exit status and that stops the program.

1. **The program runs in a process group of its own.** On Unix, the start
   puts the program into a new group. A stop sends `SIGTERM` and then
   `SIGCONT` to the group, so a stopped program also receives the request.
   After the grace, or after the program ended, the stop sends `SIGKILL` to
   the group. A drop of the handle sends `SIGKILL` to the group. The crate
   signals the group only while it has not collected the status of the
   program, because the operating system keeps the identifier of a group for
   that time.

2. **The program receives the foreground of the terminal.** When the caller
   is in the foreground group of its controlling terminal, the start makes
   the group of the program the foreground group. The handle makes the group
   of the caller the foreground group again when the program ended, when a
   stop ended it, and when a drop killed it. The handle gives back only a
   terminal that it lent. While it changes the foreground group from the
   background, it blocks `SIGTTOU` in the calling thread. The block is per
   thread and ends with the call, so no disposition of the process changes.

3. **Signal isolation follows from the foreground, and stays in the crate.**
   The terminal sends Ctrl+C, Ctrl+\\, and Ctrl+Z to its foreground group.
   That group is the group of the program, so these signals reach the program
   alone. The caller changes no disposition, and it has nothing to restore
   after an error. A consumer handles a signal from another source, such as
   `kill`, as it does without a program.

4. **The handle follows a stop of the program.** A Ctrl+Z stops the group of
   the program and not the caller, and the terminal stays with a stopped
   group. While a wait or a stop runs, the handle watches the program. It
   takes the terminal back, and it stops its own process with the same
   signal. When the shell continues the caller, the handle gives the terminal
   to the program again if the caller is in the foreground, and it continues
   the group of the program. A `SIGTTIN` or `SIGTTOU` that arrives while the
   group of the program has the foreground is a read before the start lent
   the terminal, and the handle continues the program.

5. **Windows states a limit.** Windows has no process groups of this kind,
   and every process on a console receives Ctrl+C. The start connects the
   streams, and a stop or a drop kills the program at once. The kill reaches
   the program alone, and a program that the program started survives it. A
   job object would reach the whole tree, but it needs unsafe code, which the
   lints of this repository deny.

The run of ADR-011 stays as it is. It starts the program in the group of the
caller, and its stop signals the program alone.

## Alternatives

We considered these alternatives and rejected them for the reasons below.

### The Group of the Caller, With Ignored Signals

Git runs its editor and its pager in the process group of Git, and it ignores
`SIGINT` and `SIGQUIT` while it waits. The program has the foreground without
a change, and Ctrl+Z stops both processes. But a stop can then signal
the program alone, because a signal to the group also reaches the caller. A
grandchild survives the stop. A change of a disposition also needs unsafe
code, which the lints of this repository deny, and Tokio cannot remove a
handler that it installed.

### A Group of Its Own, Without the Foreground

A new group lets the stop reach every descendant, and it needs no change of
the terminal. But a program outside the foreground group that reads the
terminal receives `SIGTTIN` and stops. Every editor and every pager reads the
terminal, so this start cannot run the programs that it exists for.

### A Session of Its Own

A new session also puts the program into a new group. But a new session has
no controlling terminal, so the program cannot open `/dev/tty`, and the
terminal sends it no signal at all.

### The Foreground Change in the Child

A shell gives the foreground to a job in the child as well, before the call
that starts the program. This closes the window in which the program reads
the terminal before it has the foreground. But that hook is unsafe code, which the
lints of this repository deny. The rule of the fourth point covers that window instead.

## Consequences

- A consumer runs an editor, a pager, or `ssh` without a run loop of its own,
  and a stop or a drop leaves no process that holds the terminal.
- A consumer needs no handler for Ctrl+C while the program runs, and it does
  not cancel its work when the user presses Ctrl+C in the program.
- The crate depends on `nix` for a block of a signal per thread and for a
  signal to the calling thread, which `rustix` does not offer without
  unsafe code. Tokio gains its `signal` feature, because the handle wakes on
  `SIGCHLD`.
- The handle follows a Ctrl+Z only while a wait or a stop runs. A program
  that stops while no call runs keeps the terminal until the next call.
- The handle watches the program that it started, and not its descendants. A
  descendant that stops alone does not stop the caller.
- The caller must not read the terminal while the program holds it, because
  the caller is then in the background and receives `SIGTTIN`.
- The crate does not save the modes of the terminal. A program that a stop
  kills can leave the terminal in raw mode, and the consumer restores it.
- A survivor of a program that ended by itself keeps running. The handle
  kills the group only on a stop or a drop.
- The Windows behavior ships as design and documentation, because no consumer
  has Windows CI.

[adr-011]: 011-process-crate.md
[clawless]: https://github.com/aonyx-ai/clawless
[clawless#798]: https://github.com/aonyx-ai/clawless/pull/798
