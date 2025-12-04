use regex::Regex;

/// Returns the regex used to match the relevant tailwind classes that need to be modified
pub fn get_classes_regex() -> Regex {
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
        "space",
        "space\\-x",
        "space\\-y",
    ]
    .map(|s| String::from(s));

    let negative_properties = bidirectional_properties
        .iter()
        .map(|prop| format!(r"\-{prop}"))
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

    Regex::new(&exp).unwrap_or_else(|err| {
        eprintln!("--spacing declaration regex not valid lollll\n{err:?}");
        panic!()
    })
}

pub fn get_spacing_declaration_regex() -> Regex {
    Regex::new(r"--spacing:\s*((\d+(?:\.\d+)?)(px|rem))").unwrap_or_else(|err| {
        eprintln!("--spacing declaration regex not valid lollll\n{err:?}");
        panic!()
    })
}
