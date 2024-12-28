# RREMIND

Simple service to remind you, much simpler than -- but inspired by -- Diane Skoll's wonderful tool [remind](https://dianne.skoll.ca/projects/remind/)

## Introduction

Pick a folder to store your appointments and the dates you would like to be reminded of. Populate this folder with text files whose names end in ".rem".

These files can contain **weekly dates**...

``Mon AT 17:00 DURATION 1 MSG Jour Fix with John Dee``, or

``Mon AT 17:00-18:00, Jour Fix with John Dee``.


... or **yearly dates** ...

``jan 1 AT 11:00 DURATION 3 MSG Clean up after new year's party``, or

``jan 1 AT 11:00-14:00, Clean up after new year's party``, or

``nov 6 REM Heiko's birthday``

... or **specific dates** ...

``2025 May 17 AT 10:00 DURATION 20 REM Give Annika her birthday present``

(Duration can be hours or minutes; values > 8 are interpreted as minutes; time is optional)

``rremind`` will ask for the location of your .rem-files on the first run -- or the location can be altered later using ``rremind config``.

You can then retrieve dates of specific days or periods, or find appointments by name (e.g. 'dentist').

## Installation

``Cargo install rremind``

**Archlinux**

``yay -S rremind``

## Usage

``rremind``

Arguments:\
  <args>
    Integer value of day relative to today;

    A range of two integers (e.g. -1..2) to list all appointments between yesterday and the day of tomorrow (inclusive)

    A date in ISO or German date format; 

    'when' followed by a search term;

    'when_was' followed by a search term (to look through the archive);

    'check' to check if all .rem-filed can be interpreted

    'config' to change the settings

    'archive' to archive all appointments that are in the past

    'version' to get the version number of your installation

    'help' for a brief help message.

    Without any arguments, ``rremind`` will show today's appointments.

## Future

Next steps:

[ ] Make appointments "taggable" and filter listings by tag and by file.\
    [ ] Color code appointments/tags/files\
    [ ] Special tag `nonblocking` meaning that appointments can still go in that slot\
    [ ] `rremind free 2 hours` to make rremind suggest appointments that take two house\
    [ ] Invent mechanism to manage doodles and the likes (i.e. block dates and unblock when date is fixed).
[ ] Make "archive" accept a parameter specifying how old the appointments need to be for archiving.\
[ ] Make archive configurable so that it collects all archives in *one* file.\
[ ] "sort" command to sort the rem files chronologically.\
[ ] "put" command to add appointments.\
[ ] Accept date-range input: rremind 0..7 to show appointments of the next 7 days.\
[ ] Make date format more flexible.\
[ ] Add nicely formatted output (for terminal).\
[ ] Add .deb package.
