excfinder - fast log parser to extract exception stacktraces
============================================================

Log files usually contain a single line per log entry.  Every such
line usually begins with a stable pattern, for example the entry's
timestamp.  On the other hand, exception stacktraces typically cover
multiple lines and have a format distinct from regular log lines.
Looking for exception stacktraces involves either searching for some
regular expression or, more simply, looking for subsequent lines which
do _not_ match a regular log line entry.  `excfinder` was written to
automate the latter approach.


Building
--------

Since `excfinder` uses [Cargo](http://crates.io), compiling it
involves running `cargo build --release` in the project directory.
The final binary can then be found under `target/release/excfinder`.


Usage
-----

`excfinder` accepts one or multiple log/text files as its arguments.
It filters these files for subsequent lines not matching the regex for
a "regular" log entry. This regexe is defined by the `-e` command line
options and defaults to `YYYY-MM-DD` at the beginning of line.

Every such identified block is written to stdout separated by a blank
line.  When having the output piped to `less` this allows to quickly
jump from one block to another by searching for `^$`.

Invoking `excfinder` with the `-h` reveals all supported command line
options.


Limitations
-----------

Right now, `excfinder` supports only valid UTF-8 input.


License
-------

`excfinder` is distributed under the MIT license.
