# RREMIND

Simple service to remind you, much simpler than -- but inspired by -- Diane Skoll's wonderful tool [remind](https://dianne.skoll.ca/projects/remind/)

## Introduction

Pick a folder to store your appointments and the dates you would like to be reminded of. Populate this folder with text files whose names end in ".rem".

These files can contain **weekly dates**...

``Mon AT 17:00 DURATION 1 MSG Jour Fix with John Dee``

... or **yearly dates** ...

``jan 1 AT 11:00 DURATION 3 MSG Clean up after new year's party``

``nov 6 REM Heiko's birthday``

... or **specific dates** ...

``2024 November 5 AT 10:00 DURATION 20 REM Buy myself a birthday present``

(Duration can be hours or minutes; values > 8 are interpreted as minutes; time is optional)

``rremind`` will ask for the location of your .rem-files on the first run.

You can then retrieve dates of specific days, or find appointments by name (e.g. 'dentist').

## Installation

``Cargo install rremind``

**Archlinux**

``yay -S rremind``

## Usage

``rremind``

Arguments:\
  <args>
    Integer value of day relative to today;

    Or 'when' followed by a search term. 

    Without any arguments, ``rremind`` will show today's appointments.

## Future

Next steps:

[ ] Make location of .rem files configurable (after first run)
[ ] Include diagnosis and help functions
[ ] Error tolerant parsing of dates
