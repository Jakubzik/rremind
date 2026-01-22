use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

use crate::{get_rremind_folders, get_user_input};

pub(crate) fn remove_line_from_file(file_name: &str, line: &str) {
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

pub(crate) fn append_line_to_file(s_file: &str, s_line: &str) -> bool {
    let mut file = OpenOptions::new().append(true).open(s_file).unwrap();

    if let Err(e) = write!(file, "\n{}", s_line) {
        eprintln!("Couldn't write to file: {}", e);
        return false;
    };

    true
}

pub(crate) fn ask_option(question: &str, options: Vec<String>) -> String {
    for (index, option) in options.iter().enumerate() {
        println!("{index} -- {option}");
    }
    println!("");
    let s = get_user_input(question, &"").trim().to_lowercase();
    match s.parse::<usize>() {
        Ok(u) => match options.get(u) {
            Some(response) => return response.to_string(),
            None => panic!("Not understood, @todo needs programming"),
        },
        Err(e) => panic!("@todo, needs programming {e}"),
    }
}

pub(crate) fn get_rrem_file_options() -> Vec<String> {
    let s_rremind_folder = get_rremind_folders();
    let mut files: Vec<String> = vec![];
    let dir = fs::read_dir(&s_rremind_folder.dir_rem_files).expect(&format!(
        "Cannot find folder >{}<, which is supposedly containing the reminder archive. Try `rremind config`?",
        s_rremind_folder.dir_rem_archive));

    for rr_file in dir {
        if let Ok(datei) = rr_file {
            let as_str = datei.path().to_str().unwrap().to_owned();
            let file_name = Path::new(&as_str).file_name().unwrap();
            if as_str.ends_with(&".rem") {
                files.push(file_name.to_string_lossy().to_string());
            }
        }
    }
    files.sort();
    files
}
