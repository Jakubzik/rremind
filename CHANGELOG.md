# version 0.0.11

- Bugfix missing time indication in some cases

# version 0.0.10

- new option "when_was" to search the archive for a keyword

# version 0.0.9

- parametrized version in code (so that `rremind version` shows the *correct* version :-), 

- corrected layout glitches in manpage

# version 0.0.8

- simplified date format of duration to 9:00-10:00; the previous syntax (9:00 DURATION 1) remains valid, though.

- make `archive` say what it has archived.

- allow "," instead of "MSG" or "REM" to introduce comments.

# version 0.0.7

- Bugfix (1): `rremind <DATE>` did not work with yearly dates.

- Bugfix (2): version showed old version.

# version 0.0.6

- `rremind <DATE>` now lists appointments of <DATE> (DATE can be ISO or German format).

# version 0.0.5

- fixed bug that made rremind print the archive folder at every call

- added "version" command

- added syntax "rremind -3..8" 

# version 0.0.4

- implemented rudimentary editing of config file

- implemented archiving

# version 0.0.3

- corrected issues with upper and lower case token words

- repaired display of duration (and calculation of appointment end)

- re-wrote the logic for looking through the appointments as preparation for an archive

# version 0.0.2

- implemented `rremind check` to go through the calendar files and report lines that cannot be interpreted.

- implemented `rremind help`

# version 0.0.1

- Created "remind" clone in Rust for fun and because "Unrecognized command; interpreting as REM" flooded my terminal

# Unreleased

[ ] BUG: entry "2025 feb 26 at 13.00 DURATION 1 MSG Whatever" does not show the time
[ ] look at <https://markwhen.com/> and compatibility
[ ] get rid of the necessity to add "at" before time indication.
[ ] make configurable templates for list of appointments (i.e. $date $from $to $subject $file)
[ ] produce simple integration test
[ ] add tag "#blocking" in order to later create a method to look for 'free' periods
[ ] Make "archive" accept a parameter specifying how old the appointments need to be for archiving.
[ ] Make archive configurable so that it collects all archives in *one* file.
[ ] Make "when" and "when_was" accept word lists (with and/or?)
[ ] "sort" command to sort the rem files chronologically.
[ ] "put" command to add appointments.
[ ] Make date format more flexible.
[ ] Add nicely formatted output (for terminal).
[ ] Add .deb package.

# History

[x] List appointments of specific date [v 0.0.6]
[x] Implement "version" command. [v 0.0.5]
[x] Make path to .rem-folder mutable [v 0.0.4]
[x] Bug: "When" fails to give the time of the appointment [v 0.0.4]
[x] archive dates that are past [v 0.0.4]
[x] Accept date-range input: rremind 0..7 to show appointments of the next 7 days. [v0.0.5]
[x] Make archive *report* what it's doing [v0.0.8]
[x] simplify format (2025 July 10, 13:15-14.15, do something) [v0.0.8]
[x] allow a comma (",") to introduce comments. `MSG` and `REM` remain valid, though [v0.0.8]
[x] repair example format on manpage [v0.0.9]
[x] repair erroneous display of `version` [v0.0.9]
[x] Make "when" work on archive [v0.0.10]
