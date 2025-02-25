mod parser;

use chrono::{Datelike, Utc};
pub(crate) use chrono::{Days, NaiveDate};
use parser::{as_date, between, get_termin_from_line, is_date};
use std::{
    env::{self},
    fs::{self, DirEntry, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
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
const VERSION: &str = "0.0.11";

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
                let thresh_date = Utc::now()
                    .date_naive()
                    .checked_sub_days(Days::new(ARCHIVE_THRESHOLD as u64))
                    .unwrap();
                thresh_date > dtm
            } else {
                false
            }
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
        args,
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
        println!("- rremind archive: archive appointments that have a specific date in the past");
        return;
    }

    if cmd == Command::EditConfig {
        edit_config();
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

    for path in directory_with_remind_files {
        if let Ok(datei) = path {
            let as_str = datei.path().to_str().unwrap().to_owned();
            if as_str.ends_with(&".rem")
                || (cmd == Command::SearchArchive && as_str.ends_with("done"))
            {
                let termine_aus_datei = std::fs::read_to_string(datei.path()).unwrap();
                match cmd {
                    Command::Archive => archive_appointments(
                        &datei,
                        &termine_aus_datei,
                        &s_rremind_folder.dir_rem_archive,
                    ),
                    Command::ListAppointments => accumulate_termine(
                        requested_date_start,
                        &termine_aus_datei,
                        &mut accumulated_termine,
                    ),
                    Command::MultiListAppointments => {
                        let mut iter_date = requested_date_start;
                        while iter_date <= requested_date_stop {
                            accumulate_termine(
                                iter_date,
                                &termine_aus_datei,
                                &mut accumulated_termine,
                            );
                            add_or_subtract_days(&mut iter_date, 1);
                        }
                    }
                    Command::SearchAppointments | Command::SearchArchive => {
                        accumulate_termine_by_search(
                            &search_term,
                            &termine_aus_datei,
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
        if acc_errors.len() == 0 {
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
        println!("- {}{}", t.appointment_description, get_zeitangabe(&t),);
    }
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
    add_line_to_archive_file(archive_name, line);
    remove_line_from_file(file_name.path().as_os_str().to_str().unwrap(), line);
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

fn remove_line_from_file(file_name: &str, line: &str) {
    //Scope to ensure that the files are closed
    let s_tmp_file_path = format!("{}.tmp", &file_name);
    let file: File =
        File::open(&file_name).expect(&format!("Cannot open rremind-file `{}`", &file_name));
    let out_file: File = File::create(&s_tmp_file_path)
        .expect(&format!("Cannot create tmp-file `{}`", &s_tmp_file_path));

    let reader = BufReader::new(&file);
    let mut writer = BufWriter::new(&out_file);

    for sline in reader.lines() {
        let single_line = sline.as_ref().unwrap();
        if single_line != line {
            writeln!(writer, "{}", single_line).unwrap_or_default();
        }
    }
    fs::rename(&s_tmp_file_path, file_name).expect(&format!("Cannot replace original rremind file `{file_name}` with temporary (new) version `{s_tmp_file_path}`"));
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
    termine: &mut Vec<Appointment>,
) {
    for line in termine_aus_datei.lines() {
        if let Some(termin_match) = get_termin_from_line(&line, Some(datum)) {
            // <-- // @todo Nov 17, 2024: Really? Some(datum)? Doing this simply to compile
            if termin_match.appointment_date == Some(datum) {
                termine.push(termin_match);
            }
        }
    }
}

fn accumulate_termine_by_search(
    search: &String,
    termine_aus_datei: &str,
    termine: &mut Vec<Appointment>,
) {
    for line in termine_aus_datei.lines() {
        if line.contains(search) {
            if let Some(found) = get_termin_from_line(&line, None) {
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
    args: Vec<String>,
    datum_start: &mut NaiveDate,
    datum_stop: &mut NaiveDate,
    search: &mut String,
) -> Command {
    if args.len() > 1 {
        if args.get(1).unwrap().contains("..") {
            let from_to = args.get(1).unwrap().split_once("..").unwrap();
            if let (Ok(days_start), Ok(days_stop)) =
                (from_to.0.parse::<i64>(), from_to.1.parse::<i64>())
            {
                add_or_subtract_days(datum_start, days_start);
                add_or_subtract_days(datum_stop, days_stop);
                return Command::MultiListAppointments;
            } else {
                // Command not intelligible
                return Command::Unknown;
            }
        } else {
            if let Ok(days) = args.get(1).unwrap().parse::<i64>() {
                add_or_subtract_days(datum_start, days);
                return Command::ListAppointments;
            } else {
                if args.get(1).unwrap() == "when" {
                    *search = args.get(2).expect(
                        "If you're calling 'when', you need a second parameter. `rremind when dentist`",
                    ).to_string();
                    return Command::SearchAppointments;
                }
                if args.get(1).unwrap() == "when_was" {
                    *search = args.get(2).expect(
                        "If you're calling 'when_was', you need a second parameter. `rremind when_was Christmas`",
                    ).to_string();
                    return Command::SearchArchive;
                }
            }
        }

        if is_date(args.get(1).unwrap()) {
            *datum_start = as_date(args.get(1).unwrap()).unwrap();
            return Command::ListAppointments;
        }
        if args.get(1).unwrap() == "version" {
            return Command::Version;
        }
        if args.get(1).unwrap() == "check" {
            return Command::Check;
        }
        if args.get(1).unwrap() == "archive" {
            return Command::Archive;
        }
        if args.get(1).unwrap() == "config" {
            return Command::EditConfig;
        }
        if args.get(1).unwrap() == "help" {
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
