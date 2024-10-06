mod parser;

use crate::parser::extract_datum_text;
use crate::parser::{
    extract_description, extract_start_time, extract_stop_time, get_full_month, get_short_month,
};
pub(crate) use chrono::{Datelike, Days, NaiveDate};
use parser::get_termin_from_line;
use std::{
    env::{self},
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

// @todo:
// - make source code English throughout

const EXIT_CODE_NO_HOME_DIR: i32 = 1;

#[derive(Debug, PartialEq)]
struct Termin {
    appointment_date: Option<chrono::NaiveDate>,
    appointment_is_yearly: bool,
    appointment_start: Option<chrono::NaiveTime>,
    appointment_stop: Option<chrono::NaiveTime>,
    appointment_description: String,
    appointment_date_alt_text: String,
}

#[derive(PartialEq, Eq)]
enum Command {
    Help,
    ListAppointments,
    SearchAppointments,
    Check,
    Unknown,
}

fn main() {
    let s_rremind_folder = get_rremind_folder(); // from config or from user.
    let mut search_term = "".to_string(); // in case this is a 'when?' request.

    let args: Vec<String> = env::args().collect();

    // @todo: expect msg should contain info on how to change the folder settings.
    let directory_with_remind_files = fs::read_dir(&s_rremind_folder).expect(&format!(
        "Cannot find folder >{}<, which is supposedly containing the reminder files.",
        s_rremind_folder
    ));

    // Starting with today, the requested
    // date is calculated:
    // rremind 1 is tomorrow, rremind -1 is yesterday, etc.
    let mut requested_date: NaiveDate = chrono::offset::Local::now().date_naive();

    let cmd = read_user_input(args, &mut requested_date, &mut search_term);

    if cmd == Command::Unknown {
        println!("Sorry, I do not understand this command. Try 'rremind help' or 'man rremind'.");
    }
    if cmd == Command::Help {
        return;
    }

    let mut accumulated_termine: Vec<Termin> = vec![];
    let mut acc_errors: Vec<String> = vec![];

    for path in directory_with_remind_files {
        if let Ok(datei) = path {
            if datei.path().to_str().unwrap().ends_with(&".rem") {
                let termine_aus_datei = std::fs::read_to_string(datei.path()).unwrap();
                match cmd {
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
    // Suche nach Notation yyyy jmonth day AT time [DURATION] ... MSG
    let year_full_month = format!(
        "{} {} {} ", // Leerzeiten damit 2024 oct 2 != 2024 oct 21
        datum.year(),
        get_full_month(datum.month()),
        datum.day()
    );

    // Suche nach Notation yyyy oct 3 AT ...
    let year_short_month = format!(
        "{} {} {} ",
        datum.year(),
        get_short_month(datum.month()),
        datum.day()
    );

    let weekly = format!("{:?} ", datum.weekday());

    let yearly = format!("{} {} ", get_short_month(datum.month()), datum.day());

    for line in termine_aus_datei.lines() {
        if line.starts_with(&year_full_month)
            || line.starts_with(&year_short_month)
            || line.starts_with(&weekly)
            || line.starts_with(&yearly)
        {
            termine.push(Termin {
                appointment_date: Some(datum),
                appointment_start: extract_start_time(&line), // <- @todo
                appointment_stop: extract_stop_time(&line),   // <- @todo
                appointment_description: extract_description(&line),
                appointment_date_alt_text: "".to_string(),
                appointment_is_yearly: false, // <- @todo
            })
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
            termine.push(Termin {
                appointment_date: None,
                appointment_date_alt_text: extract_datum_text(&line),
                appointment_start: extract_start_time(&line), // <- @todo
                appointment_stop: extract_stop_time(&line),   // <- @todo
                appointment_description: extract_description(&line),
                appointment_is_yearly: false, // <- @todo
            })
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
        if args.get(1).unwrap() == "help" {
            println!("Help for rremind:");
            println!("- rremind check: looks for syntax errors in your rremind files.");
            println!("- rremind <n>: lists appointments n days from (or to) today.");
            println!("- rremind when <term>: lists future appointments containing 'term'");
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
fn get_rremind_folder() -> String {
    if let Ok(value) = std::env::var("HOME") {
        let cf = format!("{}/.config/rremind/rr.rc", &value);
        let home_dir = Path::new(&cf);
        if home_dir.exists() {
            let cfile = std::fs::read_to_string(home_dir).unwrap();
            return cfile.split_once("=").unwrap().1.trim_end().to_string();
        } else {
            return get_rremind_folder_from_user(home_dir, &cf);
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
fn get_rremind_folder_from_user(home_dir: &Path, cf: &str) -> String {
    let mut line = String::new();
    println!("Please enter the folder containing the `rremind` files:");
    io::stdin()
        .read_line(&mut line)
        .expect("No directory entered.");
    let config_contents = format!("rremind_files={}", line);
    fs::create_dir_all(home_dir.parent().unwrap()).expect("Directory cannot be created.");
    let mut f = File::create_new(&cf).unwrap();
    f.write(&config_contents.as_bytes())
        .expect("Configuration could not be written.");

    line
}

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
