mod color;
mod parser;
mod utils;

use chrono::{Datelike, Utc};
pub(crate) use chrono::{Days, NaiveDate};
use color::*;
use parser::{as_date, between, get_termin_from_line, is_date};
use std::{
    env::{self},
    fs::{self, DirEntry, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::{
    parser::is_day,
    parser::is_month,
    utils::{append_line_to_file, ask_option, get_rrem_file_options, remove_line_from_file},
};

/// Simple version of Diane Skoll's `remind` tool
/// Looks through files with suffix ".rem" in
/// the "rremind-folder".
/// Files contain appointments in these formats
/// 2024 December 1 AT 11:00 DURATION 1 MSG Breakfast at Tiffany's
/// 2024 Dec 1 AT 11:00 DURATION 1 MSG Breakfast at Tiffany's
/// Dec 6 AT 8:00 DURATION 1 MSG Nikolaus
/// Mon AT 8:00 DURATION 1 MSG Start new week

// If we're not on Linux, don't bother
const EXIT_CODE_NO_HOME_DIR: i32 = 1;
const ARCHIVE_THRESHOLD: usize = 1; // @todo make threshold configurable
const VERSION: &str = "0.0.17";

#[derive(Debug)]
struct RRemindFolders {
    dir_rem_files: String,
    dir_rem_archive: String,
    #[allow(dead_code)]
    b_merge_archive: bool, // Should archived appointments be merged into one file? Or kept in separate files?
}

impl RRemindFolders {
    /// Write configuration to file, currently only
    /// rremind_files=<directory>
    /// rremind_archive=<directory>
    fn to_config_file_text(&self) -> String {
        format!(
            "rremind_files={}\nrremind_archive={}\n", // <- final line break is important for later parsing
            self.dir_rem_files, self.dir_rem_archive
        )
    }
}

#[derive(Debug, PartialEq)]
struct Appointment {
    appointment_date: Option<chrono::NaiveDate>,
    appointment_is_full_date: bool,
    appointment_start: Option<chrono::NaiveTime>,
    appointment_stop: Option<chrono::NaiveTime>,
    appointment_description: String,
    appointment_date_alt_text: String,
    appointment_color: Option<String>, // new in 0.15, color is defined on top of file, e.g. #color=red
}

impl Appointment {
    // Needed for archiving, so it's defensive:
    // the appointment is 'past' if and *only* if
    // we understand its date, and the date is
    // at least a day ago.
    fn is_past(&self) -> bool {
        // Birthdays and other appointments that contain no year
        // are periodical, and never 'past'
        if !self.appointment_is_full_date {
            return false;
        } else {
            if let Some(dtm) = self.appointment_date {
                match Utc::now()
                    .date_naive()
                    .checked_sub_days(Days::new(ARCHIVE_THRESHOLD as u64))
                {
                    Some(d) => d > dtm,
                    _ => panic!("Cannot construct day of today."),
                }
            } else {
                false
            }
        }
    }

    fn get_time(&self) -> String {
        if self.appointment_start.is_none() {
            return "".to_string();
        }
        if self.appointment_stop.is_none() {
            return self.appointment_start.unwrap().format("%H:%M").to_string();
        }
        return format!(
            "{} - {}",
            self.appointment_start.unwrap().format("%H:%M"),
            self.appointment_stop.unwrap().format("%H:%M")
        );
    }

    pub fn to_rrem_string(&self) -> String {
        if self.appointment_is_full_date {
            return format!(
                "{} {}, {}",
                self.appointment_date
                    .unwrap()
                    .format("%d.%m.%Y")
                    .to_string(),
                self.get_time(),
                self.appointment_description
            );
        } else {
            // panic!("{:?}", self);
            return format!("{}", self.appointment_date_alt_text);
            // println!(
            //     "SORRY, currently only full dates can be saved through the cli. (You can still note periodical dates in rrem-files, though -- see `man rremind`)"
            // );
            // return String::from("");
        }
    }
}

#[derive(PartialEq, Eq)]
enum Command {
    Help,
    ListAppointments,
    MultiListAppointments, // of a range of days
    SearchAppointments,
    SearchArchive, // new in 0.0.10
    Check,
    EditConfig,
    AddAppointment,
    Archive,
    Version,
    Unknown,
}

// See also @todo 1
// Implementing Archive: the method should
// - check if the file .rem exists in the archive-folder with the suffix .done
// - append to this file if it exists
// - create the file if it does not yet exist.
// - print the earliest date per file that was archived
// - consider writing an 'undo' (with the earliest date as input)
fn main() {
    let s_rremind_folder = get_rremind_folders(); // from config or from user.
    let mut search_term = "".to_string(); // in case this is a 'when?' request.

    let args: Vec<String> = env::args().collect();

    // Starting with today, the requested
    // date is calculated:
    // rremind 1 is tomorrow, rremind -1 is yesterday, etc.
    let mut requested_date_start: NaiveDate = chrono::offset::Local::now().date_naive();
    let mut requested_date_stop: NaiveDate = chrono::offset::Local::now().date_naive();

    let cmd = read_user_input(
        &args,
        &mut requested_date_start,
        &mut requested_date_stop,
        &mut search_term,
    );

    if cmd == Command::Unknown {
        println!("Sorry, I do not understand this command. Try 'rremind help' or 'man rremind'.");
        return;
    }
    if cmd == Command::Version {
        println!("Program Version: {}", &VERSION);
        return;
    }
    if cmd == Command::Help {
        println!("Help for rremind:");
        println!(
            "- rremind check: looks for lines in your rremind files that cannot be interpreted."
        );
        println!("- rremind <n>: lists appointments n days from (or to) today.");
        println!(
            "- rremind <dtm>: lists appointments on date <dtm>; dtm is either iso or German format (either 2025-3-10 or 10.3.2025)."
        );
        println!(
            "- rremind <n..m>: lists appointments from n days relative to today to m days relative to today (rremind -1..2 lists appointments from yesterday to the day after tomorrow)."
        );
        println!("- rremind when <term>: lists future appointments containing 'term'");
        println!(
            "- rremind when_was <term>: lists appointments containing 'term' in *archived* files"
        );
        println!("- rremind check: report syntax errors in .rem-files.");
        println!("- rremind config: edit folders");
        println!(
            "- rremind add: add an appointment. Seperate date indication from description with a comma (e.g. `rremind add Mon 13:00, lunch`)"
        );
        println!("- rremind archive: archive appointments that have a specific date in the past");
        return;
    }

    if cmd == Command::EditConfig {
        edit_config();
        return;
    }

    if cmd == Command::AddAppointment {
        add_appointment(&args[2..].join(" ").to_owned());
        return;
    }

    let mut accumulated_termine: Vec<Appointment> = vec![];
    let mut acc_errors: Vec<String> = vec![];

    let directory_with_remind_files = match cmd == Command::SearchArchive{
        false => fs::read_dir(&s_rremind_folder.dir_rem_files).expect(&format!(
            "Cannot find folder >{}<, which is supposedly containing the reminder files. Try `rremind config`?",
            s_rremind_folder.dir_rem_files
            )),
        true => {
            fs::read_dir(&s_rremind_folder.dir_rem_archive).expect(&format!(
                "Cannot find folder >{}<, which is supposedly containing the reminder archive. Try `rremind config`?",
                s_rremind_folder.dir_rem_archive))
            }
        };

    fn get_color(s_in: &str) -> Option<String> {
        let candidate = &s_in[s_in.find("=").unwrap_or_default() + 1..].to_lowercase()[..];
        match candidate {
            "red" => Some(COLOR_BRIGHT_RED.to_owned()),
            "yellow" => Some(COLOR_YELLOW.to_owned()),
            "blue" => Some(COLOR_BLUE.to_owned()),
            "cyan" => Some(COLOR_BRIGHT_CYAN.to_owned()),
            "green" => Some(COLOR_GREEN.to_owned()),
            _ => None,
        }
    }

    for path in directory_with_remind_files {
        if let Ok(datei) = path {
            let as_str = datei.path().to_str().unwrap().to_owned();
            if as_str.ends_with(&".rem")
                || (cmd == Command::SearchArchive && as_str.ends_with("done"))
            {
                let termine_aus_datei = std::fs::read_to_string(datei.path()).unwrap();
                let color = match termine_aus_datei.starts_with("# color=") {
                    true => get_color(termine_aus_datei.lines().rev().last().unwrap()),
                    false => None,
                };

                match cmd {
                    Command::Archive => archive_appointments(
                        &datei,
                        &termine_aus_datei,
                        &s_rremind_folder.dir_rem_archive,
                    ),
                    Command::ListAppointments => accumulate_termine(
                        requested_date_start,
                        &termine_aus_datei,
                        color,
                        &mut accumulated_termine,
                    ),
                    Command::MultiListAppointments => {
                        let mut iter_date = requested_date_start;
                        while iter_date <= requested_date_stop {
                            accumulate_termine(
                                iter_date,
                                &termine_aus_datei,
                                color.to_owned(),
                                &mut accumulated_termine,
                            );
                            add_or_subtract_days(&mut iter_date, 1);
                        }
                    }
                    Command::SearchAppointments | Command::SearchArchive => {
                        accumulate_termine_by_search(
                            &search_term,
                            &termine_aus_datei,
                            color.to_owned(),
                            &mut accumulated_termine,
                        )
                    }
                    Command::Check => accumulate_syntax_errors(
                        datei.path().to_str().unwrap(),
                        &termine_aus_datei,
                        &mut acc_errors,
                    ),
                    _ => panic! {"Command misunderstood, sorry."},
                }
            }
        }
    }

    if cmd == Command::Check {
        if acc_errors.is_empty() {
            println!("Check complete: all ok!");
        } else {
            for problem in acc_errors {
                println!("Problem: \n{}\n", problem);
            }
        }
        return;
    }

    sortiere(&mut accumulated_termine);

    let mut new_date: Option<NaiveDate> = None;
    println!("\n");
    for t in accumulated_termine {
        if let Some(dtm) = t.appointment_date {
            if new_date.is_none() {
                new_date = Some(dtm);
                println!(
                    "  {} ({})\n  ================",
                    dtm.to_string(),
                    dtm.weekday()
                );
            } else {
                if new_date.unwrap() != dtm {
                    new_date = Some(dtm);
                    println!(
                        "\n  {} ({})\n  ================",
                        dtm.to_string(),
                        dtm.weekday()
                    );
                }
            }
        }
        if t.appointment_color.is_some() {
            println!(
                "{}- {}{}{}",
                t.appointment_color.as_ref().unwrap(),
                t.appointment_description,
                get_zeitangabe(&t),
                COLOR_RESET
            );
        } else {
            println!("- {}{}", t.appointment_description, get_zeitangabe(&t),);
        }
    }
}

fn seems_parseable(s_info: &str) -> Option<bool> {
    let words: Vec<&str> = s_info.split_whitespace().collect();
    if is_day(words.get(0)?) {
        return Some(true);
    } // <- courageous or stupid?
    if is_month(words.get(0)?) {
        return Some(true);
    } // <- courageous or stupid?
    None
}
fn add_appointment(s_info: &str) {
    // println!("Adding this: >{s_info}<");
    let f_path = format!(
        "{}/{}",
        get_rremind_folders().dir_rem_files,
        ask_option(
            "In which file should the appointment go? ",
            get_rrem_file_options(),
        )
    );

    match get_termin_from_line(s_info, None) {
        Some(app) => {
            if app.appointment_is_full_date {
                append_line_to_file(&f_path, &app.to_rrem_string());
            } else {
                if let Some(b) = seems_parseable(s_info) {
                    if b {
                        append_line_to_file(&f_path, s_info);
                    }
                } else {
                    panic!("Sorry, I didn't understand your input.");
                }
            }
        }

        None => {
            if let Some(b) = seems_parseable(s_info) {
                if b {
                    append_line_to_file(&f_path, s_info);
                }
            } else {
                panic!("Sorry, I didn't understand your input.");
            }
        }
    };

    // dbg!("{}", app);

    // let d = ask("Which date? ", None);
    // let app_date = parse_user_input(d).unwrap();

    // let desc = ask("What's the appointments text? ");

    // add_appointment(&f_path, normalize(&app_date, &desc));
    println!("OK")
}

fn archive_appointments(file_name: &DirEntry, contents: &str, archiv_folder: &str) {
    std::fs::create_dir_all(&archiv_folder).expect(&format!(
        "Cannot create archive directory `{archiv_folder}`"
    ));

    let binding = file_name.path();

    let archive_name = format!(
        "{}{}.done",
        &archiv_folder,
        &binding.file_stem().unwrap().to_str().unwrap()
    );

    println!(
        "\nLooking for items to archive in {}...",
        file_name.path().as_os_str().to_str().unwrap()
    );

    let mut b_found = false;
    for line in contents.lines() {
        match get_termin_from_line(&line, None) {
            Some(termin) => {
                if termin.is_past() {
                    println!(" - {line}");
                    b_found = true;
                    archive_appointment(&line, &file_name, &archive_name);
                }
            }
            _ => {} // No interpretable line -> no action
        }
    }

    if !b_found {
        println!(" - no items to archive");
    }
}

fn archive_appointment(line: &str, file_name: &&DirEntry, archive_name: &str) {
    if append_line_to_file(archive_name, line) {
        remove_line_from_file(file_name.path().as_os_str().to_str().unwrap(), line);
    }
}

fn add_line_to_archive_file(archive_name: &str, line: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(archive_name)
        .unwrap();

    if let Err(e) = writeln!(file, "{}", line) {
        println!("Couldn't write to file: {}", e);
    }
}

/// Ask for directory containing the .rem-files,
/// and for directory containing the archive.
///
/// There will *not* be the option to name several
/// .rem-file directories: this would make archiving
/// more complicated, and you can achieve the same through
/// symlinks
///
/// @todo: Optionally, archived appointments should be merged
/// into *one* archive file, though.
///
/// @todo: Ask for ARCHIVE_THRESHOLD (how many days ago do
/// appointments need to be scheduled in order to be archived?)
fn edit_config() -> RRemindFolders {
    let mut home_dir = String::from("");

    if let Ok(value) = std::env::var("HOME") {
        home_dir = format!("{}/.config/rremind/rr.rc", &value);
    } else {
        println!("There's no $HOME set. Sorry, I cannot function under these circumstances");
    }

    let s_folders = get_rremind_folders();
    let s = s_folders.dir_rem_files;

    let line_dir = get_user_input(
        "Please enter the folder containing the `rremind` files:",
        &s,
    );

    let line_archive = get_user_input(
        "Please enter the folder containing the `rremind` ARCHIVE:",
        &s_folders.dir_rem_archive,
    );

    let ret = RRemindFolders {
        dir_rem_files: line_dir.trim().to_string(),
        dir_rem_archive: line_archive.trim().to_string(),
        b_merge_archive: false, // <- @todo
    };

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&home_dir)
        .unwrap();
    f.write(ret.to_config_file_text().as_bytes())
        .expect("Configuration could not be written.");

    ret
}

// Ask the terminal user the question `question`. If
// default is not empty, tell them that this is the default.
// Return the user's response.
fn get_user_input(question: &str, default: &str) -> String {
    println!("{}", question);

    if !default.is_empty() {
        println!("(Empty for `{}`)", &default);
    }

    let mut line = String::from("");

    io::stdin()
        .read_line(&mut line)
        .expect("Something went wrong trying to read your input"); // @todo

    if line.trim().is_empty() {
        return default.to_string();
    } else {
        return line;
    }
}

fn accumulate_syntax_errors(pfad: &str, termine_aus_datei: &str, acc_errors: &mut Vec<String>) {
    for line in termine_aus_datei.lines() {
        if !line.is_empty() && get_termin_from_line(&line, None).is_none() {
            acc_errors.push(format!("File: '{}':\nLine: {}\n", pfad, line));
        }
    }
}

fn accumulate_termine(
    datum: chrono::NaiveDate,
    termine_aus_datei: &str,
    color: Option<String>,
    termine: &mut Vec<Appointment>,
) {
    for line in termine_aus_datei.lines() {
        if let Some(mut termin_match) = get_termin_from_line(&line, Some(datum)) {
            // <-- // @todo Nov 17, 2024: Really? Some(datum)? Doing this simply to compile
            termin_match.appointment_color = color.to_owned();
            if termin_match.appointment_date == Some(datum) {
                termine.push(termin_match);
            }
        }
    }
}

fn accumulate_termine_by_search(
    search: &String,
    termine_aus_datei: &str,
    color: Option<String>,
    termine: &mut Vec<Appointment>,
) {
    for line in termine_aus_datei.lines() {
        if line.contains(search) {
            if let Some(mut found) = get_termin_from_line(&line, None) {
                found.appointment_color = color.clone();
                termine.push(found);
            }
        }
    }
}

// @todo: unwrap
fn add_or_subtract_days(datum: &mut NaiveDate, days: i64) {
    if days > 0 {
        *datum = datum.checked_add_days(Days::new(days as u64)).unwrap();
    } else {
        if days < 0 {
            *datum = datum
                .checked_sub_days(Days::new(days.abs() as u64))
                .unwrap();
        }
    }
}

fn read_user_input(
    args: &Vec<String>,
    datum_start: &mut NaiveDate,
    datum_stop: &mut NaiveDate,
    search: &mut String,
) -> Command {
    // We'll interpret none, one or two arguments.
    // (The first argument in the array is the path to the rremind binary)
    let (argument1, argument2) = match args.as_slice() {
        [_] => (&"".to_string(), &"".to_string()),
        [_, arg1] => (arg1, &"".to_string()),
        [_, arg1, arg2] => (arg1, arg2),
        [_, arg1, arg2, ..] => (arg1, arg2), // "add" allows n params
        _ => panic!("Please call with none, one or two parameters, see `help` or manpage"),
    };

    // Without argument, we list today's appointments
    if argument1.is_empty() {
        return Command::ListAppointments;
    }

    if argument1.trim().to_lowercase() == "add" {
        return Command::AddAppointment;
    }

    if argument1.contains("..") {
        match argument1.split_once("..") {
            Some((from, to)) => {
                if let (Ok(days_start), Ok(days_stop)) = (from.parse::<i64>(), to.parse::<i64>()) {
                    add_or_subtract_days(datum_start, days_start);
                    add_or_subtract_days(datum_stop, days_stop);
                    return Command::MultiListAppointments;
                } else {
                    // Command not intelligible
                    // Maybe hint that parse i error?
                    return Command::Unknown;
                }
            }
            None => return Command::Unknown,
        }
    }

    if argument2.is_empty()
        && let Ok(days) = argument1.parse::<i64>()
    {
        add_or_subtract_days(datum_start, days);
        return Command::ListAppointments;
    } else {
        if argument1 == "when" {
            if !argument2.is_empty() {
                *search = argument2.to_owned();
            } else {
                panic!(
                    "If you're calling 'when', you need a second parameter. `rremind when dentist`"
                );
            }
            return Command::SearchAppointments;
        }
        if argument2 == "when_was" {
            if !argument2.is_empty() {
                *search = argument2.to_owned();
            } else {
                panic!(
                    "If you're calling 'when', you need a second parameter. `rremind when dentist`"
                );
            }
            return Command::SearchArchive;
        }
    }

    if !argument1.is_empty() {
        if is_date(argument1) {
            *datum_start = as_date(argument1).unwrap();
            return Command::ListAppointments;
        }
        if argument1 == "version" {
            return Command::Version;
        }
        if argument1 == "check" {
            return Command::Check;
        }
        if argument1 == "archive" {
            return Command::Archive;
        }
        if argument1 == "config" {
            return Command::EditConfig;
        }
        if argument1 == "help" {
            return Command::Help;
        }

        return Command::Unknown;
    }
    return Command::ListAppointments;
}

/// Get path to folder containing the .rem-files.
/// We're assuming Linux and the existence of $HOME,
/// if there's no home, we'll exit.
///
/// panics if the files are not as expected
fn get_rremind_folders() -> RRemindFolders {
    if let Ok(value) = std::env::var("HOME") {
        let cf = format!("{}/.config/rremind/rr.rc", &value);
        let home_dir = Path::new(&cf);
        if home_dir.exists() {
            let cfile = std::fs::read_to_string(home_dir).unwrap();
            let dir_rem_files = between(&cfile, "rremind_files=", "\n").to_string();
            let dir_rem_archive = between(&cfile, "rremind_archive=", "\n").to_string();
            RRemindFolders {
                dir_rem_files,
                dir_rem_archive,
                b_merge_archive: false, // @todo
            }
        } else {
            fs::create_dir_all(home_dir.parent().unwrap()).expect("Directory cannot be created.");
            let _ = File::create_new(&cf).unwrap();
            // return get_rremind_folders_from_user(home_dir, &cf);
            return edit_config();
        }
    } else {
        println!(
            "What OS is this? Cannot find $HOME, which I need to locate the configuration file"
        );
        std::process::exit(EXIT_CODE_NO_HOME_DIR);
    }
}

fn get_zeitangabe(termin: &Appointment) -> String {
    if termin.appointment_start.is_none() {
        return "".to_string();
    }
    if termin.appointment_stop.is_none() {
        return format!(
            " ({} Uhr)",
            termin.appointment_start.unwrap().format("%H.%M")
        )
        .to_string();
    } else {
        return format!(
            " ({}-{} Uhr)",
            termin.appointment_start.unwrap().format("%H.%M"),
            termin.appointment_stop.unwrap().format("%H.%M")
        )
        .to_string();
    }
}

fn sortiere(accumulated_termine: &mut [Appointment]) {
    accumulated_termine.sort_by(|t1, t2| {
        if t1.appointment_start.is_none() && t2.appointment_stop.is_none() {
            return t1.appointment_date.cmp(&t2.appointment_date);
        }
        if t1.appointment_date == t2.appointment_date {
            return t1.appointment_start.cmp(&t2.appointment_start);
        }
        return t1.appointment_date.cmp(&t2.appointment_date);
    });
}
