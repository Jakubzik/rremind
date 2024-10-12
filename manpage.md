---
title: RREMIND
section: 1
header: User Manual
footer: rremind 0.0.4
date: Oktober 12, 2024
---
# NAME

rremind -- a reminder service inspired by Diane Skoll's `remind`

# SYNOPSIS

**rremind** [*OPTION*]

# DESCRIPTION

**rremind** reads through all *.rem files in the configuration folder (which is specified in rremind's first run and then stored in ~/.config/rremind/rr.rc).

Without parameter, "rremind" will list today's appointments.

With an integer parameter i, "rremind" will list the appointments i days relative to today. (`rremind -1` will show yesterday's appointments, `rremind 2` will list the appointments on the day after tomorrow).

With the parameter "when" plus a search string, rremind will list the appointments whose description contains the search string (`rremind when dentist` will list the appointments that contain the word "dentist").

# OPTIONS

**i**, 
: list appointments relative to today (i is an integer, e.g. -2 for the day before yesterday)

**when <searchterm>**,
: list future appointments containing the search-term.

**help**,
: show brief help message

**check**,
: read through the *.rem files in the configuration folder and report lines that cannot be properly interpreted by `rremind`.

**config**,
: enter or alter the directories where the remind-files are located, and where they are archived. The configuration file can equally well be edited manually (look under $HOME/rremind/rr.rc)

**archive**,
: archive all appointments that are in the past. This concerns only appointments that are specified with a full date -- periodical entries are not archived. Lines in .rem-files containing past appointments are erased from these files, and appended to files in the archive directory (see 'config' above). Files in the archive directory have the same name as the original .rem-file, but their suffix is .done (rather than .rem).

# EXAMPLES

**rremind**
: list all of today's appointments

**rremind 3**
: list all appointments three days from today

**rremind -1**
: list all of yesterday's appointments

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
