---
title: RREMIND
section: 1
header: User Manual
footer: rremind #RREMINDVERSION#
date: #DATUM#
---
# NAME

rremind -- a reminder service inspired by Diane Skoll's remind

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

# EXAMPLES

**rremind**
: list all of today's appointments

**rremind 3**
: list all appointments three days from today

**rremind -1**
: list all of yesterday's appointments

**rremind when dendist**
: list appointments that contain the word "dentist"

# AUTHORS

Written by Heiko Jakubzik, <heiko.jakubzik@shj-online.de>

# BUGS

Submit bug reports online at: <https://github.com/Jakubzik/rremind>.

# SEE ALSO

Full documentation and sources at: <https://github.com/Jakubzik/rremind>.
