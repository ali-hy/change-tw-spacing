use std::{fmt::Display, str::FromStr};

use regex::Regex;

enum CssLengthUnit {
    Px,
    Rem,
}

struct CssLength {
    coefficient: f64,
    unit: CssLengthUnit,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseLengthError;

impl FromStr for CssLength {
    type Err = ParseLengthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = get_length_regex();
        let captures = re.captures(s);

        if captures.is_none() {
            eprintln!("\"{s}\" cannot be parsed to CssLength");
            return Err(ParseLengthError);
        }

        let groups = captures.unwrap();
        let coefficient = groups.get(1).unwrap().as_str().parse::<f64>().unwrap();
        let unit = match groups.get(2).unwrap().as_str() {
            "px" => CssLengthUnit::Px,
            "rem" => CssLengthUnit::Rem,
            _ => {
                eprintln!("\"{s}\" cannot be parsed to a CssLength");
                panic!("S")
            }
        };

        Ok(CssLength { coefficient, unit })
    }
}

impl Display for CssLengthUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Px => "px",
            Self::Rem => "rem"
        })
    }
}

impl Display for CssLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.coefficient, self.unit)
    }
}

/// Returns the regex used to parse css_length.
///
/// It has 2 capture groups:
/// 1. The number part of the length
/// 2. The unit of the length
pub fn get_length_regex() -> Regex {
    Regex::new(r"^(\d+(?:\.\d+)?)(px|rem)$")
        .expect("Regex pattern for reading css length is Invalid")
}

#[allow(dead_code)]
/// Parses string that represents a CssLength to a CssLength with the unit `rem`.
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
pub fn length_to_rem(css_length_str: &str) -> f64 {
    use CssLengthUnit::*;

    let original_length: CssLength = css_length_str.parse().unwrap_or_else(|err| {
        eprintln!("{err:?}");
        panic!();
    });

    match original_length.unit {
        Px => px_to_rem(original_length.coefficient),
        Rem => original_length.coefficient
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
    let spacing_unit = captured.get(2).unwrap().as_str();

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
