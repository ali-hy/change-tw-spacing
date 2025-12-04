mod tests;
mod files;
mod values;
mod search;

use std::{env, path::PathBuf, process::exit};
use files::{find_curr_spacing, get_tw_files, update_spacing};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    // println!("{:?}", args);

    let directory = PathBuf::from(&args[1]);
    let target_spacing_arg = &args[2];

    let mut css_files: Vec<PathBuf> = Vec::new();
    let mut tw_files: Vec<PathBuf> = Vec::new();

    get_tw_files(&directory, &mut css_files, &mut tw_files).unwrap_or_else(|err| {
        eprintln!("Error occurred while getting tailwind files: {:?}", err);
        exit(1);
    });

    let (css_config_files, current_spacing) = find_curr_spacing(&css_files);
    println!(
        "\
Found --spacing declarations in the following files: {css_config_files:?}
Current spacing (in rem): {current_spacing}"
    );

    tw_files.extend(css_files);

    match update_spacing(current_spacing, target_spacing_arg, &css_config_files, &tw_files) {
        Ok((classes_updated, files_updated)) => println!(
            "Successfully updated {files_updated} files and {classes_updated} classes that use spacing!"
        ),
        Err(e) => panic!("Error occurred while updating spacing\n\n{e:?}"),
    };
}
