use regex::Regex;

pub fn get_length_regex() -> Regex {
    Regex::new(r"^(\d+(\.\d+)?)(px|rem)$")
        .expect("Regex pattern for reading css length is Invalid")
}

#[allow(dead_code)]
/// Parses string that expresses a css_length to its value in rem.
///
/// *If the length is in px it will be converted to rem with the conversion rate specified by `get_rem_to_px`*
///
/// TODO: This function should return a Result to improve reusability
///
/// ### Example
/// ```rust
/// let target_spacing_arg = &args[2]; // User entered argument. example: 4px
/// let target_spacing_rem = length_to_rem(target_spacing_arg); // Value in rem. example follow-up: 0.25rem
/// ```
pub fn length_to_rem(css_length: &str) -> f64 {
    let re = get_length_regex();

    let captured = re.captures(css_length).expect(&format!(
        "length_to_rem: param (css_length: &str = \"{css_length}\") is not a valid css length"
    ));

    let spacing_coef: f64 = captured.get(1).unwrap().as_str().parse().unwrap();
    let spacing_unit = captured.get(3).unwrap().as_str();

    match spacing_unit {
        "rem" => spacing_coef,
        "px" => px_to_rem(spacing_coef),
        _ => panic!("Unexpected unit found for --spacing (invalid unit: {spacing_unit})"),
    }
}

#[allow(dead_code)]
/// Parses string that expresses a css_length to its value in px.
///
/// *If the length is in px it will be converted to rem with the conversion rate specified by `get_rem_to_px`*
///
/// TODO: This function should return a Result to improve reusability
///
/// ### Example
/// ```rust
/// let target_spacing_arg = &args[2]; // User entered argument. example: 4px
/// let target_spacing_rem = length_to_rem(target_spacing_arg); // Value in rem. example follow-up: 0.25rem
/// ```
pub fn length_to_px(css_length: &str) -> i32 {
    let re = get_length_regex();

    let captured = re.captures(css_length).expect(&format!(
        "length_to_rem: param (css_length: &str = \"{css_length}\") is not a valid css length"
    ));

    let spacing_coef: f64 = captured.get(1).unwrap().as_str().parse().unwrap();
    let spacing_unit = captured.get(3).unwrap().as_str();

    match spacing_unit {
        "rem" => rem_to_px(spacing_coef),
        "px" => spacing_coef as i32,
        _ => panic!("Unexpected unit found for --spacing (invalid unit: {spacing_unit})"),
    }
}

// Unit conversions
/// Returns the conversion rate for rem to px.
/// By default, there are
pub fn get_rem_to_px() -> f64 {
    16.0
}

/// Converts a value from rem to px
pub fn rem_to_px(rem: f64) -> i32 {
    (rem * get_rem_to_px()) as i32
}

#[allow(dead_code)]
/// Converts a value from px to rem
pub fn px_to_rem(px: f64) -> f64 {
    px / get_rem_to_px()
}
