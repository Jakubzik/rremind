---
title: RREMIND
section: 1
header: User Manual
footer: rremind 0.0.15
date: Oktober 27, 2025
---
# NAME

rremind -- a reminder service inspired by Diane Skoll's `remind`

# SYNOPSIS

**rremind** [*OPTION*]

# DESCRIPTION

**rremind** reads through all *.rem files in the configuration folder (which is specified in rremind's first run and then stored in ~/.config/rremind/rr.rc).

Without parameter, "rremind" will list today's appointments.

With a date as parameter, "rremind" will list this date's appointments. Date can be in ISO or German format (i.e. 2025-3-10 or 10.3.2025).

With an integer parameter `i`, "rremind" will list the appointments `i` days relative to today. (`rremind -1` will show yesterday's appointments, `rremind 2` will list the appointments on the day after tomorrow).

With the parameter `when` plus a search string, rremind will list the appointments whose description contains the search string (`rremind when dentist` will list the appointments that contain the word "dentist").

Color coding: if your file `birthdays.rem` contains the first line `# color=yellow` and `work.rem` the first line `# color=red`, all birthdays will appear yellow, all work-related appointments red. Available colors are red, yellow, blue, cyan, and green.

# OPTIONS

**i**,
: list appointments relative to today (i is an integer, e.g. -2 for the day before yesterday)

**n..m**, 
: list appointments for the specified range relative to today (n, m are integerr, use e.g. -2..1 to list all appointments from the day before yesterday until and including tomorrow)

**date**,
: list appointments on the given date. Date format is ISO (2025-4-25) or German (25.4.2024).

**when [searchterm]**,
: list future appointments containing the search-term. Only appointments with a specific date are retrieved.

**when_was [searchterm]**,
: list past appointments containing the search-term (by looking through the archive). Only appointments with specific dates are retrieved.

**help**,
: show brief help message

**check**,
: read through the *.rem files in the configuration folder and report lines that cannot be properly interpreted by `rremind`.

**config**,
: enter or alter the directories where the remind-files are located, and where they are archived. The configuration file can equally well be edited manually (look under $HOME/rremind/rr.rc)

**archive**,
: archive all appointments that are in the past. This affects only appointments that are specified with a full date -- periodical entries are not archived. Lines in .rem-files containing past appointments are erased from these files, and appended to files in the archive directory (see 'config' above). Files in the archive directory have the same name as the original .rem-file, but the suffix .done (rather than .rem).

# SYNTAX OF REM-FILE ENTRIES

**yearly dates**
: [Month] [DAY] [AT optional time], INFO; for example: `Nov 24, John's birthday`, or `Nov 24 REM John's birthday`, or `Nov 24 at 22:00, John's birthday party`

**weekly dates**
: [Day] [AT optional time], INFO; for example `Mon at 18:00, dancing lesson`;

**specific dates**
: [Year] [Month] [DAY] [AT optional time], INFO; for example 2024 dec 24, Christmas.

**time indication**
: The syntax for time notation is 24-hour format rather than am/pm, e.g. 16:15. Hour and minutes can be separated either with a colon (16:15) or with a period (16.15).

# EXAMPLES

**rremind**
: list all of today's appointments

**rremind 2025-10-1**
: list all appointments on the first of October in 2025.

**rremind 10.1.2025**
: list all appointments on the first of October in 2025.

**rremind 3**
: list all appointments three days from today

**rremind -1**
: list all of yesterday's appointments

**rremind 0..7**
: list all appointment for today and the coming 7 days

**rremind when dentist**
: list appointments that contain the word "dentist"

**rremind check**
: syntax-check the files in the rremind folder

**rremind config**
: start a rudimentary dialog to set the .rem-file directory and an archive directory

**rremind archive**
: move all appointments that are past to the archive.

# AUTHORS

Written by Heiko Jakubzik, <heiko.jakubzik@shj-online.de>

# BUGS

Submit bug reports online at: <https://github.com/Jakubzik/rremind>.

# SEE ALSO

Full documentation and sources at: <https://github.com/Jakubzik/rremind>.
