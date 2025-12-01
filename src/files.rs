use crate::values::length_to_px;

use regex::Regex;
use std::{
    fs::{self, OpenOptions},
    io::{self, Error, ErrorKind, Write},
    panic,
    path::PathBuf,
};
use tempfile::NamedTempFile;

pub fn should_ignore_dir(dir_name: &str) -> bool {
    dir_name.starts_with(".")
        || ["node_modules", "dist", "build"]
            .iter()
            .any(|ignore| dir_name == *ignore)
}

pub fn is_tw_file(file_name: &str) -> bool {
    [".tsx", ".ts", ".js", ".jsx", ".html", ".scss", ".css"]
        .iter()
        .any(|suff| file_name.ends_with(suff))
}

/// This function searches for all files that (might) be using tailwind.
/// .css, .html, .ts, .tsx, .js, .jsx are all located
///
/// ### Returns
/// `Result<(Vec<String>, Vec<String>), Error>`
///
/// The tupple contains two vectors. The first is a vector of all css files. The second is a vector of all other files.
/// This distinction is to help find the css file that defines the spacing as --spacing
pub fn get_tw_files(
    dir: &String,
    css_res: &mut Vec<PathBuf>,
    res: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    let dir_entries = fs::read_dir(dir)?;

    for _entry in dir_entries {
        let entry = match _entry {
            Ok(entry) => entry,
            Err(e) => panic!(
                "DirEntry has an error. Please try to figure this out, lol.\nError: {:?}",
                e
            ),
        };

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let dir = entry.path().into_os_string().into_string().unwrap();
            if !should_ignore_dir(&entry.file_name().into_string().unwrap()) {
                get_tw_files(&dir, css_res, res)?;
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let file_name = match entry.file_name().into_string() {
            Ok(v) => v,
            Err(os_str) => {
                panic!("File name {:?} cannot be processed as a String", os_str)
            }
        };

        if file_name.ends_with(".css") {
            css_res.push(entry.path());
        } else if is_tw_file(&file_name) {
            res.push(entry.path());
        }
    }

    Ok(())
}

/// ### Returns
/// A tuple where:
/// `tuple.0` -> A vector of paths to the files with `--spacing` definitions
/// `tuple.1` -> The value of the current spacing in `px`
pub fn find_curr_spacing(css_files: &Vec<PathBuf>) -> (Vec<PathBuf>, i32) {
    let mut res = (vec![], 0);

    for curr_file in css_files {
        let file_content = fs::read_to_string(curr_file).unwrap_or(String::from(""));
        let re = Regex::new(r"--spacing:\s*((\d+(?:\.\d+)?)(px|rem))")
            .expect("Regex for --spacing is invalid");

        let captured = re.captures(&file_content);
        if captured.is_none() {
            continue;
        }

        let spacing = length_to_px(captured.unwrap().get(1).unwrap().as_str());

        if res.1 != 0 && res.1 != spacing {
            panic!(
                "Different spacings exist in your project. Please run change-tw-spacing on each directory separately. Current file: {curr_file:?}"
            )
        }

        res.0.push(curr_file.clone());
        res.1 = spacing;
    }

    res
}

/// Runs a preflight check on all files that might be edited to make sure that they're not locked, and are editable
pub fn check_locked_files(files: &Vec<PathBuf>) {
    for file in files {
        match OpenOptions::new().write(true).open(&file) {
            Ok(_f) => (),
            Err(e) => match e.kind() {
                ErrorKind::PermissionDenied => {
                    panic!("Please make sure you have permission to write to {file:?}")
                }
                _ => panic!("An error occurred while opening {file:?}"),
            },
        }
    }

    println!("No files locked, proceeding to update spacing!")
}

fn get_classes_regex() -> Regex {
    let bidirectional_properties = [
        "m",
        "my",
        "mx",
        "mt",
        "mr",
        "mb",
        "ml",
        "ms",
        "me",
        "inset",
        "top",
        "left",
        "bottom",
        "right",
        "start",
        "end",
        "spacing",
        "spacing\\-x",
        "spacing\\-y",
    ]
    .map(|s| String::from(s));
    let negative_properties = bidirectional_properties
        .iter()
        .map(|prop| format!("-{prop}"))
        .collect::<Vec<String>>();

    let positive_only_properties = [
        "p", "py", "px", "pt", "pr", "pb", "pl", "ps", "pe", "h", "w", "max\\-h", "max\\-w",
        "min\\-h", "min\\-w", "basis", "gap", "gap\\-y", "gap\\-x", "size",
    ]
    .map(|s| String::from(s));

    let property_exp = [
        &bidirectional_properties[..],
        &negative_properties[..],
        &positive_only_properties[..],
    ]
    .concat()
    .join("|");

    let exp = format!(r"\W(?:{property_exp})\-(\d+(?:\.\d+)?)");

    println!("Using the following regex to find classes that use spacing: {exp}");

    match Regex::new(&exp) {
        Ok(r) => r,
        Err(e) => panic!("Error getting classes regex!\n\n{e}"),
    }
}

/// ### Params
/// *target_files*: &Vec<String> - vector of strings each representing the path to a file that uses tailwind classes
/// *current_spacing*: f64 - the current value (in the codebase) of the --spacing variable in rem
/// *target_spacing*: f64 - the desired value of the --spacing variable in rem (classes will now be using this number)
pub fn update_spacing(
    current_spacing: i32,
    target_spacing_arg: &str,
    css_files: &Vec<PathBuf>,
    target_files: &Vec<PathBuf>,
) -> io::Result<(i32, i32)> {
    let target_spacing = length_to_px(target_spacing_arg);

    assert_ne!(current_spacing, 0);
    assert_ne!(target_spacing, 0);

    let conversion_rate = current_spacing as f64 / target_spacing as f64;
    let classes_regex = get_classes_regex();
    let all_targets = [target_files.as_slice(), css_files.as_slice()].concat();

    let mut tmp_files: Vec<Option<NamedTempFile>> = Vec::with_capacity(all_targets.len());
    let mut classes_updated_count = 0;
    let mut files_updated_count = 0;

    check_locked_files(&all_targets);

    // Iterate over files that use tailwind classes to update all classes that use the spacing variable
    for curr_file in all_targets.iter() {
        let file_content = fs::read_to_string(&curr_file)?;
        let captures_iter = classes_regex.captures_iter(&file_content);

        let mut updated_file_content = String::with_capacity(file_content.len() + 100);
        let mut prev_end = 0;
        let mut file_updates_count = 0;

        for capture_group in captures_iter {
            if file_content.as_bytes()[capture_group.get_match().end()] == b'/' {
                continue;
            }

            let coef_match = capture_group
                .get(1)
                .expect(&format!("Couldn't get capture_group[1]: {capture_group:?}"));
            updated_file_content += &file_content[prev_end..coef_match.start()];
            prev_end = coef_match.end();

            let curr_coef = coef_match.as_str().parse::<f64>().expect(&format!(
                "Failed to parse spacing coefficient in classname `{}` (tried to parse: `{}`)",
                capture_group.get_match().as_str(),
                coef_match.as_str()
            ));

            let new_coef = curr_coef * conversion_rate;
            let new_value_str = match new_coef - new_coef.floor() {
                0.0|0.25|0.5|0.75 => new_coef.to_string(),
                _ => format!("[{}px]", curr_coef * current_spacing as f64)
            };

            updated_file_content += &new_value_str;
            file_updates_count += 1;
        }

        classes_updated_count += file_updates_count;

        // If the file is in the css_config_files array then update --spacing
        if css_files.iter().any(|f| *f == *curr_file) {
            let spacing_declaration_regex = Regex::new(r"--spacing:\s*((\d+(?:\.\d+)?)(px|rem))")
                .expect("--spacing declaration regex not valid lollll");
            let captures_iter = spacing_declaration_regex.captures_iter(&file_content);

            for capture_group in captures_iter {
                let length_match = &capture_group.get(1).unwrap();

                updated_file_content += &file_content[prev_end..length_match.start()];
                prev_end = length_match.end();
                updated_file_content += target_spacing_arg;

                file_updates_count += 1;
            }
        }

        // If any matches were found (and changes were made)
        if file_updates_count > 0 {
            updated_file_content += &file_content[prev_end..];

            let mut tmp = NamedTempFile::with_prefix_in(
                curr_file.file_stem().unwrap(),
                curr_file.parent().unwrap(),
            )
            .expect(&format!(
                "Couldn't unwrap cur_file.file_name() for {curr_file:?}"
            ));
            tmp.write(updated_file_content.as_bytes())?;
            tmp_files.push(Some(tmp));

            files_updated_count += 1;
        } else {
            tmp_files.push(None);
        };
    }

    // Replace files with tmp files
    for (curr_file, tmp) in all_targets.iter().zip(tmp_files) {
        if tmp.is_some() {
            fs::rename(tmp.unwrap(), curr_file)?;
        }
    }

    // Iterate over files that define
    Ok((classes_updated_count, files_updated_count))
}
