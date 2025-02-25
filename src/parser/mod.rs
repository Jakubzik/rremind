// Parsing Dates
// First application is a checkup of the configured files

// use core::slice::SlicePattern;
// use std::ops::Index;

use chrono::{Datelike, Days, NaiveDate, NaiveTime, TimeDelta};

use crate::Appointment;

// Start_Time and Stop_Time: 10:00 DURATION 20 is possible, make
// 10:00-10:20 possible, too
struct TimeHelper {
    start: Option<NaiveTime>,
    stop: Option<NaiveTime>,
}

const MONTHS: &'static [&'static str] = &[
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

const DAYS: &'static [&'static str] = &["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

const NO_INFO: &str =
    "Appointment contains no information (add 'msg' or 'rem' to the line to change that)";

pub(crate) fn is_date(s_text: &str) -> bool {
    as_date(s_text).is_some()
}

/// s_text can either be German or ISO format
pub(crate) fn as_date(s_text: &str) -> Option<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(s_text, "%Y-%m-%d") {
        return Some(date);
    } else {
        if let Ok(date) = NaiveDate::parse_from_str(s_text, "%d.%m.%Y") {
            return Some(date);
        }
    }
    None
}

/// Parse s_text and return a `Termin` if possible.
///
/// Appointments without year indication are mapped to the current year
///
/// Appointments without month indication return the next matching
/// weekday relative to today
///
/// If the text cannot be parsed, `None` is returned.
pub fn get_termin_from_line(s_text: &str, start_date: Option<NaiveDate>) -> Option<Appointment> {
    if let Some(r) = get_termin_from_full_date(&s_text) {
        return Some(r);
    }
    if let Some(s) = get_termin_without_month(&s_text, start_date) {
        return Some(s);
    }
    if let Some(t) = get_termin_without_year(&s_text, start_date) {
        return Some(t);
    }

    None
}

// 2024 sep 9 AT 10:00 DURATION 1 MSG Whatever
pub(crate) fn get_termin_from_full_date(s_in: &str) -> Option<Appointment> {
    let words: Vec<&str> = s_in.split_whitespace().collect();
    if let Ok(year) = words.get(0)?.parse::<i32>() {
        let month = get_month_as_no(words.get(1)?)?;
        if let Ok(day) = words.get(2)?.parse::<usize>() {
            let da = NaiveDate::from_ymd_opt(year, month as u32, day as u32);

            let t: TimeHelper = extract_duration(s_in);

            return Some(Appointment {
                appointment_date: da,
                appointment_is_full_date: true,
                // appointment_start: extract_start_time(s_in),
                // appointment_stop: extract_stop_time(s_in),
                appointment_start: t.start,
                appointment_stop: t.stop,
                appointment_description: extract_description(s_in),
                appointment_date_alt_text: extract_datum_text(s_in),
            });
        }
    }
    None
}

// Either Jan 24 2024 AT 10:00 DURATION 1, or
//        Jan 24 2024 AT 10:00-12:00
fn extract_duration(s_in: &str) -> TimeHelper {
    // See if we have AT ... - ...
    let small = s_in.to_lowercase();

    // We're looking for notations with "-" first,
    // i.e. 10:00-11:30 rather than 10:00 DURATION 90
    if let Some(index) = small.find(" at ") {
        let start = index + " at ".len();
        let end = match small.len() >= start + "10:00 - 11:00".len() {
            true => start + "10:00 - 11:00".len(),
            _ => start + "10:00-11:00".len(),
        };

        // Avoid getting in the middle of unicode bytes
        let text_vec = s_in.chars().collect::<Vec<_>>();
        let s_i = text_vec[start..end].iter().cloned().collect::<String>();

        // Is the duration indicated with "-", i.e. 10:00-11:00 ?
        if s_i.contains("-") {
            let times = s_i.split_once("-").unwrap();
            return TimeHelper {
                start: NaiveTime::parse_from_str(times.0.trim(), &get_time_parser(times.0.trim()))
                    .ok(),
                stop: get_stop(times.1.trim()),
            };
        }
    }

    // This works for "DURATION" notation
    return TimeHelper {
        start: extract_start_time(s_in),
        stop: extract_stop_time(s_in),
    };
}

// Helper to determine if we're dealing
// with "10.15" or "10:15" (both are ok)
fn get_time_parser(s_in: &str) -> String {
    match s_in.contains(":") {
        true => "%H:%M".to_string(),
        _ => "%H.%M".to_string(),
    }
}

// The end time could end like this: "-8:00 M", or "-8:00, get up"
// .get_stop will trim these to "8:00"
// Input (`excerpt`) must already be trimmed
fn get_stop(excerpt: &str) -> Option<NaiveTime> {
    match excerpt.contains(" ") {
        true => {
            let candidate = between(excerpt, "", " ");
            let parser = get_time_parser(candidate);
            NaiveTime::parse_from_str(candidate, &parser).ok()
        }
        _ => match excerpt.contains(",") {
            true => {
                let candidate = between(excerpt, "", ",");
                let parser = get_time_parser(candidate);
                NaiveTime::parse_from_str(candidate, &parser).ok()
            }
            _ => NaiveTime::parse_from_str(excerpt, "%H:%M").ok(),
        },
    }
}

//// Read yearly appointments, such as
//// jan 7 msg birthday ds
////
//// Returns a "Termin" adding the current year -- or None,
//// it his is not notation without year.
pub(crate) fn get_termin_without_year(
    s_in: &str,
    start_date: Option<NaiveDate>,
) -> Option<Appointment> {
    let words: Vec<&str> = s_in.split_whitespace().collect();
    if is_month(words.get(0)?) {
        let year = match start_date {
            None => chrono::offset::Local::now().date_naive().year(),
            Some(yr) => yr.year(),
        };
        let month = get_month_as_no(s_in)?;
        if let Ok(day) = strip_final_comma(words.get(1)?).parse::<usize>() {
            let da = NaiveDate::from_ymd_opt(year, month as u32, day as u32);

            let t: TimeHelper = extract_duration(s_in);
            return Some(Appointment {
                appointment_date: da,
                appointment_is_full_date: false,
                appointment_start: t.start,
                appointment_stop: t.stop,
                appointment_description: extract_description(s_in),
                appointment_date_alt_text: extract_datum_text(s_in),
            });
        }
    }
    None
}

// Nov 24, Sunday: I want to be able to understand "24," as "24", so
// final commas are stripped
fn strip_final_comma(get: &str) -> &str {
    if get.ends_with(",") {
        &get[0..get.len() - 1]
    } else {
        get
    }
}

//// Read weekly appointments, such as
//// Mon msg buy lunch
////
//// Returns a "Termin" adding the current year and month -- or None,
//// if s_in does not start with a weekday
pub(crate) fn get_termin_without_month(
    s_in: &str,
    start_date: Option<NaiveDate>,
) -> Option<Appointment> {
    // let small = s_in.to_lowercase();
    let words: Vec<&str> = s_in.split_whitespace().collect();
    if is_day(words.get(0)?) {
        if let Some(da) = find_next_date(words.get(0)?, start_date) {
            let t: TimeHelper = extract_duration(s_in);
            return Some(Appointment {
                appointment_date: Some(da),
                appointment_is_full_date: false,
                appointment_start: t.start,
                appointment_stop: t.stop,
                appointment_description: extract_description(s_in),
                appointment_date_alt_text: extract_datum_text(s_in),
            });
        }
    }
    None
}

/// Find the next date of this weekday
/// If today, for example, is Sat, Oct 5, 2024,
/// and weekday is "mon", the method returns
/// Oct 7, 2024
fn find_next_date(weekday: &str, start_date: Option<NaiveDate>) -> Option<NaiveDate> {
    let mut target_date = match start_date {
        None => chrono::offset::Local::now().date_naive(),
        Some(dtm) => dtm,
    };
    let wd = &weekday[0..3].to_lowercase(); // Wednesday and Wed are both ok as input
    for _ii in 0..7 {
        if target_date.weekday().to_string().to_lowercase() == *wd {
            return Some(target_date);
        }
        target_date = target_date.checked_add_days(Days::new(1)).unwrap();
    }
    None
}

fn is_month(month: &str) -> bool {
    MONTHS.iter().any(|s| month.to_lowercase().starts_with(*s))
}

fn get_month_as_no(month: &str) -> Option<usize> {
    match MONTHS
        .iter()
        .position(|mmonth| month.to_lowercase().starts_with(mmonth))
    {
        Some(i) => Some(i + 1),
        _ => None,
    }
}

fn is_day(weekday_name: &str) -> bool {
    // let wdn = &weekday_name[0..2];
    DAYS.iter()
        .any(|s| weekday_name.to_lowercase().starts_with(*s))
}

// fn get_day_as_no(weekday_name: &str) -> Option<usize> {
//     match DAYS.iter().position(|day| weekday_name.starts_with(day)) {
//         Some(i) => Some(i + 1),
//         _ => None,
//     }
// }
// Wird in der Zeile entweder durch "MSG" oder durch "REM"
// eingeleitet
// new: MSG, REM, or ","
pub(crate) fn extract_description(line: &str) -> String {
    match get_right_from(line, vec![" msg ", " rem ", ", "]) {
        Some(s) => s,
        None => NO_INFO.to_string(),
    }
}

// If there's a splitter in the `line`, the text to the
// right of the splitter is returned.
// line = "2024 Nov 1 msg what?", splitter ["this", "that", "msg"] => Some(what?)
fn get_right_from(line: &str, splitters: Vec<&str>) -> Option<String> {
    let small = line.to_lowercase();
    for splitter in splitters {
        // println!("Looking for >{splitter}< in >{small}<");
        if let Some(index) = small.find(splitter) {
            return Some(
                String::from_utf8(line.as_bytes()[index + splitter.len()..].to_vec()).unwrap(),
            );
        }
    }

    None
}

// textual representation of the date, i.e.
// Wed, or Jan 10, or 2024 Jan 10
// This is for display of an appointment
// found through the search functionality
pub(crate) fn extract_datum_text(line: &str) -> String {
    // Mon AT
    // mar 1 AT
    // 2024 sep 1 AT
    match line.split_once(" AT ") {
        Some(text) => text.0.trim().to_string(),
        _ => String::new(),
    }
}

// AT XYZ DURATION dd
pub(crate) fn extract_stop_time(line: &str) -> Option<NaiveTime> {
    let small = line.to_lowercase();
    if let Some(start) = extract_start_time(&small) {
        let duration = between(&small, " duration ", " ");
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
pub(crate) fn extract_start_time(line: &str) -> Option<NaiveTime> {
    let small = line.to_lowercase();
    // @todo: change syntax: " at " is too common to use as identifyer
    let s = between(&small, " at ", " ");
    match NaiveTime::parse_from_str(s, "%H:%M") {
        Ok(r) => Some(r),
        _ => None,
    }
    // None
    // todo!()
}

// pub(crate) fn get_full_month(monat: u32) -> String {
//     match monat {
//         1 => "January".to_string(),
//         2 => "February".to_string(),
//         3 => "March".to_string(),
//         4 => "April".to_string(),
//         5 => "May".to_string(),
//         6 => "June".to_string(),
//         7 => "July".to_string(),
//         8 => "August".to_string(),
//         9 => "September".to_string(),
//         10 => "October".to_string(),
//         11 => "November".to_string(),
//         12 => "December".to_string(),
//         _ => "UnknownMonth".to_string(),
//     }
// }

// pub(crate) fn get_short_month(monat: u32) -> String {
//     get_full_month(monat)[..3].to_lowercase()
// }

pub fn between<'a>(source: &'a str, start: &'a str, end: &'a str) -> &'a str {
    let start_position = source.find(start);
    // if start == "" {
    //     start_position = Some(0);
    // }

    if start_position.is_some() {
        let start_position = start_position.unwrap() + start.len();
        let source = &source[start_position..];
        let end_position = source.find(end).unwrap_or_default();
        return &source[..end_position];
    }
    return "";
}

#[cfg(test)]
mod test_parsing {
    // use super::*;

    use std::str::FromStr;

    use chrono::{Datelike, NaiveDate, NaiveTime, offset};

    use crate::{
        Appointment,
        parser::{
            get_month_as_no, get_termin_from_full_date, get_termin_from_line,
            get_termin_without_month, get_termin_without_year, is_date, is_month,
        },
    };

    // use super::NO_INFO;

    fn get_testtermin_thisyear() -> Appointment {
        let year = offset::Local::now().date_naive().year();
        Appointment {
            appointment_date: NaiveDate::from_ymd_opt(year, 11, 1),
            appointment_is_full_date: true,
            appointment_start: None,
            appointment_stop: None,
            appointment_description: "birthday".to_string(), //NO_INFO.to_string(),
            appointment_date_alt_text: "".to_string(),
        }
    }

    #[test]
    fn parsing() {
        let s_test = "2024-1-13";
        assert!(is_date(s_test));
        let s_test = "13.1.2024";
        assert!(is_date(s_test));
        let s_test = "13.13.2024";
        assert!(!is_date(s_test));
    }

    #[test]
    fn parsing_no_year1() {
        let s_test = "jan 6 msg birthday";
        assert_eq!(is_month(&s_test), true);
        assert_eq!(get_month_as_no(&s_test), Some(1));
    }

    #[test]
    fn parsing_no_year2() {
        // let s_test = "jan 6 msg birthday";
        let s_test = "nov 1 msg birthday";
        assert!(get_termin_without_year(&s_test, None).is_some());

        let s_test = "FeBrUARY 11 msg birthday";
        assert!(get_termin_without_year(&s_test, None).is_some());

        let s_test = "PFeBrUARYo 11 msg birthday";
        assert!(get_termin_without_year(&s_test, None).is_none());
    }

    #[test]
    fn parsing_no_year3() {
        let s_test = "nov 1 msg birthday";
        assert_eq!(
            get_termin_without_year(&s_test, None)
                .unwrap()
                .appointment_date,
            get_testtermin_thisyear().appointment_date
        );
    }

    #[test]
    fn parsing_no_year4() {
        let s_test = "asdf 1 msg birthday";
        assert!(get_termin_without_year(&s_test, None).is_none());
    }

    #[test]
    fn parsing_no_year5() {
        let s_test = "nov mon msg birthday";
        assert!(get_termin_without_year(&s_test, None).is_none());
    }

    #[test]
    fn parsing_weekly() {
        let s_test = "mon at 10:00 msg birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_without_month(&s_test, None).is_some());
    }

    #[test]
    fn parsing_weekly4() {
        let s_test = "ton at 10:00 msg birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_without_month(&s_test, None).is_none());
    }

    #[test]
    fn parsing_weekly2() {
        let s_test = "mon at 10:00 msg birthday";
        assert_eq!(
            get_termin_without_month(&s_test, None)
                .unwrap()
                .appointment_date
                .unwrap()
                .weekday()
                .to_string(),
            "Mon".to_string(),
        );

        let s_test = "Wednesday at 10:00 msg birthday";
        assert!(get_termin_without_month(&s_test, None).is_some());
        assert_eq!(
            get_termin_without_month(&s_test, None)
                .unwrap()
                .appointment_date
                .unwrap()
                .weekday()
                .to_string(),
            "Wed".to_string()
        );
        assert!(
            get_termin_without_month(&s_test, None)
                .unwrap()
                .appointment_start
                .is_some()
        );

        assert_eq!(
            get_termin_without_month(&s_test, None)
                .unwrap()
                .appointment_start
                .unwrap(),
            NaiveTime::from_str("10:00").unwrap()
        );

        assert!(
            get_termin_without_month(&s_test, None)
                .unwrap()
                .appointment_stop
                .is_none()
        );
    }

    #[test]
    fn parsing_full_date() {
        let s_test = "2024 nov 6 aT 10:00 DURATION 1 msg my birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_from_full_date(&s_test).is_some());
        assert_eq!(
            get_termin_from_full_date(&s_test)
                .unwrap()
                .appointment_date
                .unwrap()
                .month0(),
            10
        );
        assert_eq!(
            get_termin_from_full_date(&s_test)
                .unwrap()
                .appointment_start
                .unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap()
        );
    }

    #[test]
    fn parsing_comprehensive1() {
        let s_test = "2024 nov 6 aT 10:00 DURATION 1 msg my birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_from_line(&s_test, None).is_some());
        assert_eq!(
            get_termin_from_line(&s_test, None)
                .unwrap()
                .appointment_date
                .unwrap()
                .month0(),
            10
        );
    }

    #[test]
    fn parsing_comprehensive2() {
        let s_test = "nov 6 aT 10:00 DURATION 1 msg my birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_from_line(&s_test, None).is_some());
        assert_eq!(
            get_termin_from_line(&s_test, None)
                .unwrap()
                .appointment_date
                .unwrap()
                .month0(),
            10
        );
    }

    #[test]
    fn parsing_comprehensive3() {
        // assert!(is_day("Mon"));
        let s_test = "Mon aT 10:00 DURATION 1 msg my birthday";
        assert!(get_termin_from_line(&s_test, None).is_some());
        assert_eq!(
            get_termin_from_line(&s_test, None)
                .unwrap()
                .appointment_date
                .unwrap()
                .weekday()
                .to_string(),
            "Mon".to_string()
        );
    }

    #[test]
    fn parsing_comprehensive4() {
        let s_test = "Mun aT 10:00 DURATION 1 msg my birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert!(get_termin_from_line(&s_test, None).is_none());
    }

    #[test]
    fn parsing_comprehensive5() {
        let s_test = "Mon aT 10:00 DURATION 1 msg my birthday";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert_eq!(
            get_termin_from_line(&s_test, None)
                .unwrap()
                .appointment_description,
            "my birthday"
        );
    }

    #[test]
    fn parsing_comprehensive6() {
        let s_test = "Mon aT 10:00 DURATION 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        assert_eq!(
            get_termin_from_line(&s_test, None)
                .unwrap()
                .appointment_description,
            "my birthdayß"
        );
    }

    #[test]
    fn parsing_comprehensive7() {
        let s_test = "Mon aT 10:00-11:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("10:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("11:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive8() {
        let s_test = "Mon aT 10:00 - 11:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("11:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive9() {
        let s_test = "Mon aT 10:00-9:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("9:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("10:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive10() {
        let s_test = "Mon aT 10:00-9:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("9:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("10:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive11() {
        let s_test = "Mon aT 10:00 - 9:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("9:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("10:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive12() {
        let s_test = "Mon aT 1:00 - 9:00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("9:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("1:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive13() {
        let s_test = "Mon aT 1:00-9.00 1 ß, my birthdayß";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "my birthdayß");
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("9:00", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("1:00", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive14() {
        let s_test = "2024 Nov 23 AT 10.30-12:30, Kammerorchester Börsensaal Hauptprobe";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(
            tmp.appointment_description,
            "Kammerorchester Börsensaal Hauptprobe"
        );
        assert_eq!(
            tmp.appointment_stop,
            NaiveTime::parse_from_str("12:30", "%H:%M").ok()
        );
        assert_eq!(
            tmp.appointment_start,
            NaiveTime::parse_from_str("10:30", "%H:%M").ok()
        );
    }

    #[test]
    fn parsing_comprehensive15() {
        let s_test = "Nov 24, Sonntag";
        // assert!(get_termin_without_year(&s_test).is_none());
        let tmp = get_termin_from_line(&s_test, None).unwrap();
        assert_eq!(tmp.appointment_description, "Sonntag");
    }
}
