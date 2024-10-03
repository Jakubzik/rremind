use chrono::{Datelike, Days, NaiveDate, NaiveTime, TimeDelta};
use std::{
    env::{self},
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

const EXIT_CODE_NO_HOME_DIR: i32 = 1;

struct Termin {
    appointment_date: Option<chrono::NaiveDate>,
    appointment_start: Option<chrono::NaiveTime>,
    appointment_stop: Option<chrono::NaiveTime>,
    appointment_description: String,
    appointment_date_alt_text: Option<String>,
}

// @todo
// Ausgabe als Tabelle, siehe:
// <https://stackoverflow.com/questions/30379341/how-to-print-well-formatted-tables-to-the-console>
// Man-page
// Code aufhübschen
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

    read_user_input(args, &mut requested_date, &mut search_term);

    let mut accumulated_termine: Vec<Termin> = vec![];

    for path in directory_with_remind_files {
        if let Ok(datei) = path {
            if datei.path().to_str().unwrap().ends_with(&".rem") {
                let termine_aus_datei = std::fs::read_to_string(datei.path()).unwrap();
                if search_term != "" {
                    accumulate_termine_by_search(
                        &search_term,
                        termine_aus_datei,
                        &mut accumulated_termine,
                    );
                } else {
                    accumulate_termine(requested_date, termine_aus_datei, &mut accumulated_termine);
                }
            }
        }
    }

    sortiere(&mut accumulated_termine);

    for t in accumulated_termine {
        println!(
            "{} -- {}{}",
            match t.appointment_date {
                Some(dat) => dat.to_string(),
                _ => match t.appointment_date_alt_text {
                    Some(ref info) => info.clone(),
                    _ => "".to_string(),
                },
            },
            t.appointment_description,
            get_zeitangabe(&t),
        );
    }
}

fn accumulate_termine(
    datum: chrono::NaiveDate,
    termine_aus_datei: String,
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
                appointment_date_alt_text: None,
            })
        }
    }
}

fn accumulate_termine_by_search(
    search: &String,
    termine_aus_datei: String,
    termine: &mut Vec<Termin>,
) {
    for line in termine_aus_datei.lines() {
        if line.contains(search) {
            termine.push(Termin {
                appointment_date: None,
                appointment_date_alt_text: Some(extract_datum_text(&line)),
                appointment_start: extract_start_time(&line), // <- @todo
                appointment_stop: extract_stop_time(&line),   // <- @todo
                appointment_description: extract_description(&line),
            })
        }
    }
}

// textual representation of the date, i.e.
// Wed, or Jan 10, or 2024 Jan 10
// This is for display of an appointment
// found through the search functionality
fn extract_datum_text(line: &str) -> String {
    // Mon AT
    // mar 1 AT
    // 2024 sep 1 AT
    match line.split_once(" AT ") {
        Some(text) => text.0.trim().to_string(),
        _ => String::new(),
    }
}

fn read_user_input(args: Vec<String>, datum: &mut NaiveDate, search: &mut String) {
    if args.len() > 1 {
        if let Ok(days) = args.get(1).unwrap().parse::<i64>() {
            if days > 0 {
                *datum = datum.checked_add_days(Days::new(days as u64)).unwrap();
            } else {
                *datum = datum
                    .checked_sub_days(Days::new(days.abs() as u64))
                    .unwrap();
            }
        } else {
            if args.get(1).unwrap() == "when" {
                *search = args.get(2).expect(
                    "If you're calling 'when', you need a second parameter. `rremind when dentist`",
                ).to_string();
            }
        }
    }
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

// Wird in der Zeile entweder durch "MSG" oder durch "REM"
// eingeleitet
fn extract_description(line: &str) -> String {
    let tmp = line.split_once(" MSG ");
    if let Some(msg) = tmp {
        return msg.1.to_string();
    } else {
        let tmp = line.split_once(" REM ");
        if let Some(msg) = tmp {
            return msg.1.to_string();
        } else {
            return "Keine Info zum Termin".to_string();
        }
    }
}

// AT XYZ DURATION dd
fn extract_stop_time(line: &str) -> Option<chrono::NaiveTime> {
    if let Some(start) = extract_start_time(line) {
        let duration = between(line, " DURATION ", " ");
        if let Ok(f_duration) = duration.parse::<i64>() {
            if f_duration > 8 {
                // werten wir als Minuten
                let (r, _) =
                    start.overflowing_add_signed(TimeDelta::try_minutes(f_duration).unwrap());
                return Some(r);
            } else {
                // werten wir als Stunden
                let (r, _) =
                    start.overflowing_add_signed(TimeDelta::try_hours(f_duration).unwrap());

                return Some(r);
            }
        }
    }
    None
    // todo!()
}

// AT 11:00
fn extract_start_time(line: &str) -> Option<chrono::NaiveTime> {
    let s = between(line, " AT ", " ");
    match NaiveTime::parse_from_str(s, "%H:%M") {
        Ok(r) => Some(r),
        _ => None,
    }
    // None
    // todo!()
}

fn get_full_month(monat: u32) -> String {
    match monat {
        1 => "January".to_string(),
        2 => "February".to_string(),
        3 => "March".to_string(),
        4 => "April".to_string(),
        5 => "May".to_string(),
        6 => "June".to_string(),
        7 => "July".to_string(),
        8 => "August".to_string(),
        9 => "September".to_string(),
        10 => "October".to_string(),
        11 => "November".to_string(),
        12 => "December".to_string(),
        _ => "Unbekannt".to_string(),
    }
}

fn get_short_month(monat: u32) -> String {
    get_full_month(monat)[..3].to_lowercase()
}

fn between<'a>(source: &'a str, start: &'a str, end: &'a str) -> &'a str {
    let start_position = source.find(start);

    if start_position.is_some() {
        let start_position = start_position.unwrap() + start.len();
        let source = &source[start_position..];
        let end_position = source.find(end).unwrap_or_default();
        return &source[..end_position];
    }
    return "";
}
