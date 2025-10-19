//! Command Parser from an input &str
/// Run parse(input: &str) to retrieve and arguments list.
/// Use ArgList trait functions available for &[Argument] to retrieve and convert the values.
pub use core::str::FromStr;

use super::error::*;

pub use heapless::{String, Vec};

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

const READ_BUFFER_LENGTH: usize = 192;

const MAX_NUMBER_PARAMS: usize = 5;
const MAX_CMD_NAME_LENGTH: usize = 24;
const MAX_PARAM_NAME_LENGTH: usize = 16;
const MAX_VALUE_LENGTH: usize = 64;

const SEPARATOR: char = '\u{001E}';
const ESCAPE: char = '\u{005C}';
const CR: char = '\u{000D}';
const DEFAULT_CMD: &str = "help";

// ——————————————————————————————————————————— Parse —————————————————————————————————————————————

/// Takes an input str and parses it, returning an arguments list.
#[inline]
pub fn parse(input: &str) -> Result<Vec<Argument, MAX_NUMBER_PARAMS>> {
    // Creating command with args Struct. Defaulting to DEFAULT_CMD cmd.
    let mut args: Vec<Argument, MAX_NUMBER_PARAMS> = Vec::new();

    if input.is_empty() {
        return Ok(args);
    }

    let mut processed_buf: String<READ_BUFFER_LENGTH> = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    // Replacing spaces in quotes with SEPARATOR
    for char in input.chars() {
        match char {
            CR => {}
            '"' => {
                if escaped && in_quotes {
                    processed_buf.push('"').map_err(|_| Error::CommandTooLong)?;
                }
                else {
                    in_quotes = !in_quotes;
                }
                escaped = false;
            }
            ' ' if in_quotes => {
                escaped = false;
                processed_buf.push(SEPARATOR).map_err(|_| Error::CommandTooLong)?;
            }
            ESCAPE if in_quotes => {
                if escaped {
                    processed_buf.push(ESCAPE).map_err(|_| Error::CommandTooLong)?;
                    escaped = false;
                }
                else {
                    escaped = true;
                }
            }
            c if in_quotes => {
                escaped = false;
                processed_buf.push(c).map_err(|_| Error::CommandTooLong)?;
            }
            c => {
                processed_buf.push(c.to_ascii_lowercase()).map_err(|_| Error::CommandTooLong)?;
            }
        }
    }

    // Check for dangling escape character
    if escaped {
        return Err(Error::Parse("dangling escape \"\\\" char".into_truncate()));
    }

    // Check for unmatched quotes
    if in_quotes {
        return Err(Error::Parse("unmatched quotes".into_truncate()));
    }

    // ——————————————————————————————————— Processing arguments ——————————————————————————————————————

    let processed_buf = processed_buf.split_ascii_whitespace();

    for word in processed_buf {
        // Sanitizing. Orphan "=" triggers error.
        if word == "=" || word.starts_with('=') || word.ends_with('=') {
            return Err(Error::Parse("\"=\" spacing".into_truncate()));
        }

        let mut elements = word.splitn(2, '=');
        let param_str = elements.next().unwrap();
        let value_str = elements.next();

        let param = String::try_from(param_str).map_err(|_| Error::ArgTooLong)?;
        let mut value: String<MAX_VALUE_LENGTH> = String::new();

        // If param has value, we restore the space characters
        if let Some(val_) = value_str {
            for char in val_.chars() {
                let c_to_push = if char == SEPARATOR { ' ' } else { char };
                value.push(c_to_push).map_err(|_| Error::ArgTooLong)?;
            }
        }

        args.push(Argument { param, value }).map_err(|_| Error::TooManyArgs)?;
    }

    Ok(args)
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Argument
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Default, Clone)]
pub struct Argument {
    pub param: String<MAX_PARAM_NAME_LENGTH>,
    pub value: String<MAX_VALUE_LENGTH>,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

// —————————————————————————————————————————— Arg List —————————————————————————————————————————————

pub trait ArgList {
    fn get_parsed_param<T>(&self, param: &str) -> Result<T>
    where
        T: FromStr;

    fn get_str_param<'a>(&'a self, param: &str) -> Option<&'a str>;

    fn contains_param(&self, str: &str) -> bool;
}

impl ArgList for &[Argument] {
    #[inline]
    fn get_parsed_param<T>(&self, param: &str) -> Result<T>
    where
        T: FromStr,
    {
        let arg = self
            .iter()
            .find(|s| s.param.eq_ignore_ascii_case(param))
            .ok_or_else(|| Error::MissingArg(param.into_truncate()))?;

        let val_as_str = arg.value.as_str();

        let value: T = val_as_str.parse().map_err(|_| Error::Parse(param.into_truncate()))?;

        Ok(value)
    }

    #[inline]
    fn get_str_param<'a>(&'a self, param: &str) -> Option<&'a str> {
        self.iter()
            .find(|arg| arg.param.eq_ignore_ascii_case(param))
            .map(move |arg| arg.value.as_str())
    }

    #[inline]
    fn contains_param(&self, str: &str) -> bool {
        self.iter().any(|arg| arg.param.eq_ignore_ascii_case(str))
    }
}
