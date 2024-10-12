mod parser;

use chrono::Utc;
pub(crate) use chrono::{Days, NaiveDate};
use parser::{between, get_termin_from_line};
use std::{
    env::{self},
    fs::{self, DirEntry, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

// @todo:
// - make source code English throughout

const EXIT_CODE_NO_HOME_DIR: i32 = 1;

#[derive(Debug)]
struct RRemindFolders {
    dir_rem_files: String,
    dir_rem_archive: String,
    #[allow(dead_code)]
    b_merge_archive: bool, // Should archived appointments be merged into one file? Or kept in separate files?
}

impl RRemindFolders {
    fn to_config_file_text(&self) -> String {
        format!(
            "rremind_files={}\nrremind_archive={}\n", // <- final line break is important for later parsing
            self.dir_rem_files, self.dir_rem_archive
        )
    }
}

#[derive(Debug, PartialEq)]
struct Termin {
    appointment_date: Option<chrono::NaiveDate>,
    appointment_is_full_date: bool,
    appointment_start: Option<chrono::NaiveTime>,
    appointment_stop: Option<chrono::NaiveTime>,
    appointment_description: String,
    appointment_date_alt_text: String,
}
impl Termin {
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
                Utc::now().date_naive() > dtm
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
    SearchAppointments,
    Check,
    EditConfig,
    Archive,
    Unknown,
}

// See also @todo 1
// Another @todo: the tests don't run through
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
    let mut requested_date: NaiveDate = chrono::offset::Local::now().date_naive();

    let cmd = read_user_input(args, &mut requested_date, &mut search_term);

    if cmd == Command::Unknown {
        println!("Sorry, I do not understand this command. Try 'rremind help' or 'man rremind'.");
    }
    if cmd == Command::Help {
        println!("Help for rremind:");
        println!(
            "- rremind check: looks for lines in your rremind files that cannot be interpreted."
        );
        println!("- rremind <n>: lists appointments n days from (or to) today.");
        println!("- rremind when <term>: lists future appointments containing 'term'");
        println!("- rremind config: edit folders");
        println!("- rremind archive: archive appointments in the past");
        return;
    }
    // if cmd == Command::Archive {
    //     archive_appointments();
    //     return;
    // }
    if cmd == Command::EditConfig {
        edit_config();
        return;
    }

    let mut accumulated_termine: Vec<Termin> = vec![];
    let mut acc_errors: Vec<String> = vec![];

    // @todo: expect msg should contain info on how to change the folder settings.
    let directory_with_remind_files =
        fs::read_dir(&s_rremind_folder.dir_rem_files).expect(&format!(
            "Cannot find folder >{}<, which is supposedly containing the reminder files. Try `rremind config`?",
            s_rremind_folder.dir_rem_files
        ));

    // panic!("Archiv ist: {}", s_rremind_folder.dir_rem_archive);
    for path in directory_with_remind_files {
        if let Ok(datei) = path {
            if datei.path().to_str().unwrap().ends_with(&".rem") {
                let termine_aus_datei = std::fs::read_to_string(datei.path()).unwrap();
                match cmd {
                    Command::Archive => archive_appointments(
                        &datei,
                        &termine_aus_datei,
                        &s_rremind_folder.dir_rem_archive,
                    ),
                    Command::ListAppointments => accumulate_termine(
                        requested_date,
                        &termine_aus_datei,
                        &mut accumulated_termine,
                    ),
                    Command::SearchAppointments => accumulate_termine_by_search(
                        &search_term,
                        &termine_aus_datei,
                        &mut accumulated_termine,
                    ),
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

    for t in accumulated_termine {
        println!(
            "{} -- {}{}",
            match t.appointment_date {
                Some(dat) => dat.to_string(),
                _ => t.appointment_date_alt_text.to_string(),
            },
            t.appointment_description,
            get_zeitangabe(&t),
        );
    }
}

//
fn archive_appointments(file_name: &DirEntry, contents: &str, archiv_folder: &str) {
    // - convert file_name to archive-Filename
    // - loop through lines of file to see
    //   if entries are past and copy then
    std::fs::create_dir_all(&archiv_folder).expect(&format!(
        "Cannot create archive directory `{archiv_folder}`"
    ));
    let binding = file_name.path();
    // let archive_name =
    // archive_name = format!("{}.done", &archive_name[..archive_name.len() - 4]);
    let archive_name = format!(
        "{}{}.done",
        &archiv_folder,
        &binding.file_stem().unwrap().to_str().unwrap()
    );

    // println!("Archivname: {}", archive_name);
    // println!("... voll: {}{}", archiv_folder, archive_name);
    for line in contents.lines() {
        match get_termin_from_line(&line) {
            Some(termin) => {
                if termin.is_past() {
                    archive_appointment(&line, &file_name, &archive_name);
                }
            }
            _ => {}
        }
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

// Asks for folder and archive-folder
// Needs restructuring, repeated asking (read_line)
// needs extra method.
// Also: check input, implement gentle fail
// And should we allow more than one folder for input?
// @todo 1
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
        if get_termin_from_line(&line).is_none() {
            acc_errors.push(format!("File: '{}':\nLine: {}\n", pfad, line));
        }
    }
}

fn accumulate_termine(
    datum: chrono::NaiveDate,
    termine_aus_datei: &str,
    termine: &mut Vec<Termin>,
) {
    for line in termine_aus_datei.lines() {
        if let Some(termin_match) = get_termin_from_line(&line) {
            if termin_match.appointment_date == Some(datum) {
                termine.push(termin_match);
            }
        }
    }
}

fn accumulate_termine_by_search(
    search: &String,
    termine_aus_datei: &str,
    termine: &mut Vec<Termin>,
) {
    for line in termine_aus_datei.lines() {
        if line.contains(search) {
            if let Some(found) = get_termin_from_line(&line) {
                termine.push(found);
            }
        }
    }
}

fn read_user_input(args: Vec<String>, datum: &mut NaiveDate, search: &mut String) -> Command {
    if args.len() > 1 {
        if let Ok(days) = args.get(1).unwrap().parse::<i64>() {
            if days > 0 {
                *datum = datum.checked_add_days(Days::new(days as u64)).unwrap();
            } else {
                *datum = datum
                    .checked_sub_days(Days::new(days.abs() as u64))
                    .unwrap();
            }
            return Command::ListAppointments;
        } else {
            if args.get(1).unwrap() == "when" {
                *search = args.get(2).expect(
                    "If you're calling 'when', you need a second parameter. `rremind when dentist`",
                ).to_string();
                return Command::SearchAppointments;
            }
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
            println!("ARCHIVE: {}", dir_rem_archive);
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

/// Asks the user for the config-path, writes
/// the path in the config file, returns the
/// path.
// fn get_rremind_folders_from_user(home_dir: &Path, cf: &str) -> RRemindFolders {
//     let mut line = String::new();
//     println!("Please enter the folder containing the `rremind` files:");
//     io::stdin()
//         .read_line(&mut line)
//         .expect("No directory entered.");
//     let config_contents = format!("rremind_files={}", line);
// fs::create_dir_all(home_dir.parent().unwrap()).expect("Directory cannot be created.");
// let mut f = File::create_new(&cf).unwrap();
//     f.write(&config_contents.as_bytes())
//         .expect("Configuration could not be written.");

//     line
// }

fn get_zeitangabe(termin: &Termin) -> String {
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

fn sortiere(accumulated_termine: &mut [Termin]) {
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

// fn get_short_wochentag(w: Weekday) -> String {
//     match w {
//         Weekday::Mon => "Mon".to_string(),
//         Weekday::Tue => todo!(),
//         Weekday::Wed => todo!(),
//         Weekday::Thu => todo!(),
//         Weekday::Fri => todo!(),
//         Weekday::Sat => todo!(),
//         Weekday::Sun => todo!(),
//     }
// }
