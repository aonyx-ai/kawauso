# Processes for Kawauso

`kawauso-process` runs the external programs of a Kawauso application. Every
application starts a program in the same way. The caller names the program and
its arguments. The crate collects the output of the program, and it reports how
the program ended. [ADR-011] records why the crate exists and where its
boundary is.

Every requirement in this document has an identifier, and the code that
implements or tests a requirement references the identifier in a comment.
[Tracey] checks that every requirement is implemented and tested. The key
words MUST and MUST NOT have the meaning that [RFC 2119] defines.

This specification is a living document that grows with the crate.

## Invocation

An application that starts an external program names the program and its
arguments. An invocation carries this description, and it separates the
description of a command from the run of the command. An application can
therefore build a command, write it to a log, and name it in an error.

The arguments are a list, and each element of the list is one argument. No
shell reads the command, so nothing splits an argument at a space, removes a
quotation mark, or expands a character such as `*`. Some commands run in a
directory of their own, such as a build tool that works on the project in its
directory. The caller can therefore name a working directory, and most
commands need none.

A log line and an error message have to name the command that they describe,
so an invocation renders as a command line. The rendering is for a person. No
caller parses it, and no shell runs it. A space separates the words of the
line. A word that holds a space, or a word that is empty, therefore needs
marks that show where it starts and where it ends.

process[invocation.program]
An invocation MUST name the program of the command.

process[invocation.arguments]
An invocation MUST carry the arguments of the command as a list, and it MUST
keep each argument as the caller wrote it. The crate MUST NOT split, quote, or
expand an argument that it gives to the program.

process[invocation.directory]
An invocation MUST carry a working directory for the command when the caller
names one. The crate MUST NOT require a working directory.

process[invocation.display]
The crate MUST render an invocation as a command line that a person can read.
The rendering MUST name the program and every argument, in the order in which
the caller named them. It MUST show where a word of the line starts and where
it ends, when the word holds a space or when the word is empty.

## Running

An invocation describes a command, and an application then runs it. The crate
runs the command in one call. The result of the run reports how the command
ended, what the command wrote to each of its streams, and how long the run
took. The two streams stay apart, because a program separates its result from
its diagnostics.

A command that ends without success is no failure of the run. The check mode
of a formatter ends without success when it finds a file to format. That
status is the answer that the application asked for. The result therefore
carries the status, and an error reports a run that did not happen.

Three decisions apply to every command, and no caller states them. The
standard input is null, the command inherits the environment of the process
that runs it, and the operating system resolves the program. A run also reads
both streams while it waits for the end of the command. A command that fills a
pipe stops until a reader empties it, so a run that reads one stream after the
other stops with it. A run that the caller abandons ends the command as well.

A command that does not start is a failure of the run. Nothing ran, so the
crate reports an error, and the message names the command line, because the
reader of a log has to know which command did not start. A caller that must
not accept a command that failed makes a check on the result. The message of
that check names the command line, the status, and what the command wrote to
its standard error, which is where a program states why it stopped.

process[run]
The crate MUST run the command of an invocation in one call, and it MUST report
a result for the run.

process[run.exit]
The result of a run MUST carry the exit status of the command. The crate MUST
NOT return an error when a command ran and its exit status is not a success.

process[run.output]
The result of a run MUST carry what the command wrote to its standard output
and what it wrote to its standard error. It MUST keep the two apart, and it
MUST keep the output as the command wrote it.

process[run.duration]
The result of a run MUST carry the time that the run took.

process[run.drain]
The crate MUST read the standard output and the standard error of the command
while it waits for the exit. A run MUST complete when the command writes more
than the capacity of a pipe to one stream or to both streams.

process[run.stdin]
The crate MUST open the standard input of the command as null. A command that
reads the standard input MUST see the end of the input.

process[run.environment]
The command MUST inherit the environment of the process that runs it.

process[run.resolution]
The crate MUST let the operating system resolve the program of an invocation.
The crate MUST NOT search `PATH` for the program.

process[run.abandonment]
The crate MUST end a command that still runs when the caller drops the run.

process[run.error]
The crate MUST return an error, and MUST NOT panic, when the command cannot
start.

process[run.error.message]
The message of the error MUST name the command line of the invocation.

process[run.success]
The result of a run MUST offer a check that fails when the exit status of the
command is not a success. The check MUST succeed when the exit status is a
success.

process[run.success.message]
The message of a check that failed MUST name the command line of the
invocation, the exit status of the command, and what the command wrote to its
standard error.

## Streaming

A run in one call reports after the command ended. An application that shows
the progress of a command needs the output while the command runs. An
application that turns the output into events of its own needs it as well. The
crate therefore has a second form. A handle starts the command and gives the
output to the caller as lines, in the order in which the lines arrive.

A line comes from one of the two streams, and the reader has to know which
one, because a diagnostic is not a result. A line is text, because a display
takes text and not bytes. The decoding is lossy, and the capture of the run
keeps the bytes as the command wrote them. Nothing that the command wrote is
lost. A command on Windows ends a line with two characters, and a line carries
neither of them.

A caller that shows the output often waits for another event as well, such as
a timeout or a token that cancels the work. The read of a line stops when the
other event occurs first. The bytes that the read took stay with the handle.
The caller reads again, and it gets the line that the command wrote.

The handle ends the run in the same way as the one-call form, and it reports
the same result. A caller that read no line still gets the whole capture. The
two forms differ in what the caller sees during the run, and not in what the
run reports. A caller that drops the handle ends the command, because a handle
that leaves a command behind leaks a process.

process[stream]
The crate MUST offer a handle that gives the output of a command to the caller
as lines while the command runs. The handle MUST give the lines to the caller
in the order in which they arrive.

process[stream.tag]
Each line MUST name the stream that produced it.

process[stream.decode]
The crate MUST decode a line as text, and a byte that is part of no valid
character MUST become the replacement character. The capture of the run MUST
keep the bytes as the command wrote them.

process[stream.crlf]
A line MUST NOT carry the characters that end it, whether the command ends the
line with `\n` or with `\r\n`.

process[stream.cancellation]
The crate MUST NOT lose output when the caller drops a read before the read
reports a line. The read that follows MUST report every line that the command
wrote.

process[stream.wait]
A wait on the handle MUST report the result that a run in one call reports.
The result MUST hold the whole capture of both streams, whether or not the
caller read a line.

process[stream.abandonment]
The crate MUST end a command that still runs when the caller drops the handle.

[adr-011]: ../../adrs/011-process-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
