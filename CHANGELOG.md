# version 0.0.4

- implemented rudimental editing of config file

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

- Created "remind" clone in Rust for fun and because "Unrecognized command; interpreting as REM" flooded my terminal

# Unreleased

[ ] Implement "version" command.
[ ] Make "archive" accept a parameter specifying how old the appointments need to be for archiving.
[ ] Make archive configurable so that it collects all archives in *one* file.
[ ] "sort" command to sort the rem files chronologically.
[ ] "put" command to add appointments.
[ ] Accept date-range input: rremind 0..7 to show appointments of the next 7 days.
[ ] Make date format more flexible.
[ ] Add nicely formatted output (for terminal).
[ ] Add .deb package.

# History

[x] Make path to .rem-folder mutable [v 0.0.4]
[x] Bug: "When" fails to give the time of the appointment [v 0.0.4]
[x] archive dates that are past [v 0.0.4]
