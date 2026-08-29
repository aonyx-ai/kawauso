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
arguments. An invocation carries this description. It separates the description
of a command from the run of the command. An application can therefore build a
command, write it to a log, and name it in an error.

The arguments are a list, and each element of the list is one argument. The
program gets each argument as the caller wrote it. No shell reads the command,
so nothing splits an argument at a space, removes a quotation mark, or expands
a character such as `*`. An argument with a space stays one argument, and an
argument with a metacharacter reaches the program unchanged. An application
that wants a shell names the shell as the program, and gives the command line
to it as one argument.

Some commands run in a directory of their own. A build tool works on the
project in its directory, and a formatter reads the configuration file that it
finds there. The caller can therefore name a working directory for the command.
Most commands need no directory of their own, so an invocation does not require
one. A command without a working directory runs where the process runs.

A log line and an error message have to name the command that they describe. A
reader that sees `git status` knows which command failed. An invocation
therefore renders as a command line, with the program first and the arguments
after it.

The rendering is for a person. No caller parses it, and no shell runs it. It
therefore does not have to be a command line that a shell accepts.

A space separates the words of the line. A word that holds a space would
therefore read as two words, and a word that is empty would read as none. The
rendering shows where such a word starts and where it ends. The marks belong to
the line and not to the argument, and the program still receives the argument
as the caller wrote it.

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

[adr-011]: ../../adrs/011-process-crate.md
[rfc 2119]: https://www.rfc-editor.org/rfc/rfc2119
[tracey]: https://tracey.bearcove.eu/
