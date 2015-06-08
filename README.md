excfinder - fast log parser to extra exception stacktraces
==========================================================

Log files usually contain a single line per log entry.  Typically,
every such line begins with a stable pattern, for example the entry's
timestamp.  On the other hand, exception stacktraces usually cover
multiple lines and have a format which is distinct from regular log
lines.  Looking for exception stacktraces involves either searching
for some matching regular expression or, more simply, looking for
subsequent lines which do not match a regular log line entry.
`excfinder` was written to automate the latter approach.


Usage
-----

Since `excfinder` uses [Cargo](http://crates.io), compiling it
involves running `cargo build --release` in the project directory.
The finally binary can then be located as `target/release/excfinder`.

`excfinder` accepts one or more log files where it locates subsequent
lines which do not match the regular expression for a "regular" log
entry and dumps these to stdout.  The regex can be changed with the
`-e` option and defaults to matching a `YYYY-MM-DD` at the beginning
of a line.
