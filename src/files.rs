use crate::search::{get_classes_regex, get_spacing_declaration_regex};
use crate::values::length_to_px;

use regex::Regex;
use std::{
    fs::{self, OpenOptions},
    io::{self, Error, ErrorKind, Write},
    panic,
    path::PathBuf,
};
use tempfile::NamedTempFile;

/// Returns a boolean - The directory should be ignored when return is true, and should be searched when return is false
pub fn should_ignore_dir(dir_name: &str) -> bool {
    dir_name.starts_with(".")
        || ["node_modules", "dist", "build"]
            .iter()
            .any(|ignore| dir_name == *ignore)
}

/// Returns a boolean - The file should be searched when return is true, and should be ignored when return is false
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
/// The tuple contains two vectors. The first is a vector of all css files.
/// The second is a vector of all other files that should be searched for tailwind classes.
/// This distinction is to help find the css file that defines the spacing as --spacing
pub fn get_tw_files(
    dir: &PathBuf,
    css_res: &mut Vec<PathBuf>,
    res: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    let dir_entries = fs::read_dir(dir)?;

    for _entry in dir_entries {
        let entry = _entry.expect(&format!(
            "DirEntry has an error. Please try to figure this out, lol."
        ));

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if !should_ignore_dir(&entry.file_name().into_string().unwrap()) {
                get_tw_files(&dir, css_res, res)?;
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let file_name = entry.file_name().into_string().unwrap_or_else(|os_str| {
            eprintln!("File name {:?} cannot be processed as a String", os_str);
            panic!();
        });

        if file_name.ends_with(".css") {
            css_res.push(entry.path());
        } else if is_tw_file(&file_name) {
            res.push(entry.path());
        }
    }

    Ok(())
}

/// Returns a tuple `res`, where:
///
/// `res.0` -> A vector of paths to the files with `--spacing` definitions
///
/// `res.1` -> The value of the current spacing in `px`
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

/// Runs a preflight check on all files that might be edited to make sure that they're not locked, and that the user
/// has sufficient permissions to open all files involved
fn check_locked_files(files: &Vec<PathBuf>) {
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

/// Update the content of a file that uses tailwind classes
///
/// Returns
///
///
fn get_updated_content_in_tw_file(
    file_content: &str,
    current_spacing: i32,
    target_spacing: i32,
    file_updates_count: &mut i32,
) -> String {
    let conversion_rate = current_spacing as f64 / target_spacing as f64;
    let classes_regex = get_classes_regex();
    let captures_iter = classes_regex.captures_iter(&file_content);

    let mut prev_end = 0;
    let mut updated_file_content = String::with_capacity(file_content.len() + 100);

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
            0.0 | 0.25 | 0.5 | 0.75 => new_coef.to_string(),
            _ => format!("[{}px]", curr_coef * current_spacing as f64),
        };
        updated_file_content += &new_value_str;

        *file_updates_count += 1;
    }

    updated_file_content += &file_content[prev_end..];
    updated_file_content
}

/// ### Params
/// `file_content`: content of the file
///
/// `target_spacing_arg`: target value for --spacing as entered by the user
///
/// `files_updates_count`: mutable reference to a variable that gets incremented with each change in
///
/// **Returns** the updated file content for a file that might contain a css --spacing declaration
fn get_updated_content_in_css_config_file(
    file_content: &str,
    target_spacing_arg: &str,
    file_updates_count: &mut i32,
) -> String {
    let spacing_declaration_regex = get_spacing_declaration_regex();
    let captures_iter = spacing_declaration_regex.captures_iter(&file_content);

    let mut updated_file_content = String::with_capacity(file_content.len() + 100);
    let mut prev_end = 0;

    for capture_group in captures_iter {
        let length_match = &capture_group.get(1).unwrap();

        updated_file_content += &file_content[prev_end..length_match.start()];
        prev_end = length_match.end();
        updated_file_content += target_spacing_arg;

        *file_updates_count += 1;
    }
    updated_file_content += &file_content[prev_end..];

    updated_file_content
}

/// ### Params
/// `curr_file`: the file that this tmp file corresponds to
///
/// `content`: content of the new tmp file
/// **Returns** The temp file object
fn create_tmp_file(curr_file: &PathBuf, content: &str) -> NamedTempFile {
    let mut tmp =
        NamedTempFile::with_prefix_in(curr_file.file_stem().unwrap(), curr_file.parent().unwrap())
            .unwrap_or_else(|err| {
                eprintln!("Couldn't create tmp file for {curr_file:?}\n\n{err:?}");
                panic!()
            });

    tmp.write(content.as_bytes()).unwrap_or_else(|err| {
        eprintln!("Couldn't write to tmp file for {curr_file:?}\n\n{err:?}");
        panic!()
    });

    tmp
}

/// ### Params
/// `targets`: a vector of the paths that the tmp files will be saved to
///
/// `optional_tmp_files`: a vector of Option objects. When there is Some(tmp_file), the tmp_file will replace
/// its corresponding file from the `targets` vector. Otherwise, nothing happens
///
/// **Returns** a result of nothing, or an error
fn save_optional_tmp_files(
    targets: &Vec<PathBuf>,
    optional_tmp_files: &Vec<Option<NamedTempFile>>,
) -> Result<(), Error> {
    for (curr_file, tmp) in targets.iter().zip(optional_tmp_files) {
        if tmp.is_some() {
            fs::rename(tmp.as_ref().unwrap(), curr_file)?;
        }
    }

    Ok(())
}

/// ### Params
/// `current_spacing: i32` - the current value (in the codebase) of the --spacing variable in px
///
/// `target_spacing_arg: string` - the target spacing as entered by the user (including the unit)
///
/// `css_files: &Vec<PathBuf>` - vector of paths each representing the path to a file that uses tailwind classes
///
/// `target_files: &Vec<PathBuf>` - vector of paths each representing the path to a file that uses tailwind classes
///
/// ### Returns
/// `io::Result<(i32, i32)>` - A Result containing a tuple. The first element is the number of classes updated,
/// and the second element is the number of files updated.
pub fn update_spacing(
    current_spacing: i32,
    target_spacing_arg: &str,
    css_files: &Vec<PathBuf>,
    target_files: &Vec<PathBuf>,
) -> io::Result<(i32, i32)> {
    let target_spacing = length_to_px(target_spacing_arg);

    assert_ne!(current_spacing, 0);
    assert_ne!(target_spacing, 0);

    let all_targets = [target_files.as_slice(), css_files.as_slice()].concat();

    let mut tmp_files: Vec<Option<NamedTempFile>> = Vec::with_capacity(all_targets.len());
    let mut classes_updated_count = 0;
    let mut files_updated_count = 0;

    check_locked_files(&all_targets);

    // Iterate over files that use tailwind classes to update all classes that use the spacing variable
    for curr_file in all_targets.iter() {
        let file_content = fs::read_to_string(&curr_file)?;
        let mut file_updates_count = 0;

        // update tailwind classes in all target files
        let mut updated_file_content = get_updated_content_in_tw_file(
            &file_content,
            current_spacing,
            target_spacing,
            &mut file_updates_count,
        );

        classes_updated_count += file_updates_count;

        // If the file is in the css_config_files array then update --spacing
        if css_files.iter().any(|f| *f == *curr_file) {
            updated_file_content = get_updated_content_in_css_config_file(
                &updated_file_content,
                target_spacing_arg,
                &mut file_updates_count,
            );
        }

        // If any changes were made, save them to a temp file
        if file_updates_count > 0 {
            tmp_files.push(Some(create_tmp_file(curr_file, &updated_file_content)));
            files_updated_count += 1;
        } else {
            tmp_files.push(None);
        };
    }

    // Replace files with tmp files
    save_optional_tmp_files(&all_targets, &tmp_files)?;

    // Iterate over files that define
    Ok((classes_updated_count, files_updated_count))
}
