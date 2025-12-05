mod tests;
mod files;
mod css_length;
mod search;

use std::{path::PathBuf, process::exit};
use files::{find_curr_spacing, get_tw_files, update_spacing};
use clap::Parser;

/// A program to update the --spacing variable in a project that uses tailwind without
/// changing the actual spacing in the app.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The project's directory. The program will search this directory for all the relevant files
    /// that need to be read and updated.
    #[arg(short, long, default_value_t=String::from("."))]
    dir: String,

    /// The new spacing that the project should use
    target_spacing_arg: String,

    /// Integer - Specifies a conversion rate from rem to px.
    #[arg(long, default_value_t=16)]
    conversion_rate: i32,
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args);

    let directory = PathBuf::from(&args.dir);
    let target_spacing_arg = &args.target_spacing_arg;

    let mut css_files: Vec<PathBuf> = Vec::new();
    let mut tw_files: Vec<PathBuf> = Vec::new();

    get_tw_files(&directory, &mut css_files, &mut tw_files).unwrap_or_else(|err| {
        eprintln!("Error occurred while getting tailwind files: {:?}", err);
        exit(1);
    });
    println!("Got {} css tiles, and {} tw_files", css_files.len(), tw_files.len());

    let (css_config_files, current_spacing) = find_curr_spacing(&css_files);
    println!(
        "\
Found --spacing declarations in the following files: {css_config_files:?}
Current spacing (in px): {current_spacing}"
    );

    tw_files.extend(css_files);

    match update_spacing(current_spacing, target_spacing_arg, &css_config_files, &tw_files) {
        Ok((classes_updated, files_updated)) => println!(
            "Successfully updated {files_updated} files and {classes_updated} classes that use spacing!"
        ),
        Err(e) => panic!("Error occurred while updating spacing\n\n{e:?}"),
    };
}
