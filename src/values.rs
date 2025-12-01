use regex::Regex;

pub fn get_length_regex() -> Regex {
    Regex::new(r"^(\d+(\.\d+)?)(px|rem)$")
        .expect("Regex pattern for reading css length is Invalid")
}

// TODO: This function should return a Result to improve reusability
#[allow(dead_code)]
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
pub fn get_rem_to_px() -> f64 {
    16.0
}

pub fn rem_to_px(rem: f64) -> i32 {
    (rem * get_rem_to_px()) as i32
}

#[allow(dead_code)]
pub fn px_to_rem(px: f64) -> f64 {
    px / get_rem_to_px()
}
