//! Command Parser from an input &str
/// Run parse(input: &str) to generate a ParsedCommand Struct
/// Use ArgList trait functions available for &[Argument] to retrieve and convert the vales
pub use core::str::FromStr;

use super::errors::*;

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
const DEFAULT_CMD: &str = "help";

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            CmdWithArgs
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Default, Clone)]
pub struct ParsedCommand {
  pub cmd:  String<MAX_CMD_NAME_LENGTH>,
  pub args: Vec<Argument, MAX_NUMBER_PARAMS>,
}

impl ParsedCommand {
  // ——————————————————————————————————————————— Parse —————————————————————————————————————————————

  /// Takes an input str and parses it, returning a CommandWithArgs Struct.
  pub fn parse(input: &str) -> Result<Self> {
    //
    let mut processed_buf: String<READ_BUFFER_LENGTH> = String::new();
    let mut in_quotes = false;

    // Replacing spaces in quotes with SEPARATOR
    // Converting chars to lowercase
    for char in input.chars() {
      match char {
        '"' => {
          in_quotes = !in_quotes;
        },
        ' ' if in_quotes => {
          processed_buf.push(SEPARATOR).map_err(|_| CliError::ParseBuffer)?;
        },
        c if c.is_ascii_uppercase() => {
          processed_buf.push(c.to_ascii_lowercase()).map_err(|_| CliError::ParseBuffer)?;
        },
        c => {
          processed_buf.push(c).map_err(|_| CliError::ParseBuffer)?;
        },
      }
    }

    // Check for unmatched quotes
    if in_quotes {
      return Err(CliError::Parse("Unmatched Quotes".into_truncated()));
    }

    // Creating command with args Struct. Defaulting to "help" cmd.
    let mut command_with_args = ParsedCommand {
      cmd:  String::from_str(DEFAULT_CMD).unwrap(),
      args: Vec::new(),
    };

    // —————————————————————————————————— Extracting command name ————————————————————————————————————

    let mut processed_buf = processed_buf.split_ascii_whitespace();

    // Extract first element. If empty, we return the default Struct with "help" cmd.
    let Some(cmd_str) = processed_buf.next()
    else {
      return Ok(command_with_args);
    };

    command_with_args.cmd = String::try_from(cmd_str).map_err(|_| CliError::ParseBuffer)?;

    // ——————————————————————————————————— Processing arguments ——————————————————————————————————————

    for word in processed_buf {
      // Sanitizing. Orphan "=" triggers error.
      if word == "=" || word.starts_with('=') {
        return Err(CliError::Parse("\"=\" delimiter spacing".into_truncated()));
      }

      let mut elements = word.splitn(2, '=');
      let param_str = elements.next().unwrap();
      let value_str = elements.next();

      let param = String::try_from(param_str).map_err(|_| CliError::ParseBuffer)?;
      let mut value: String<MAX_VALUE_LENGTH> = String::new();

      // If param has value, we restore the space characters
      if let Some(val_) = value_str {
        for char in val_.chars() {
          let c_to_push = if char == SEPARATOR { ' ' } else { char };
          value.push(c_to_push).map_err(|_| CliError::ParseBuffer)?;
        }
      }

      command_with_args
        .args
        .push(Argument { param, value })
        .map_err(|_| CliError::ParseBuffer)?;
    }

    Ok(command_with_args)
  }
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
  fn get_parsed_param<T>(&self, param: &str) -> Result<T>
  where
    T: FromStr,
  {
    let arg = self
      .iter()
      .find(|s| s.param.eq_ignore_ascii_case(param))
      .ok_or_else(|| CliError::MissingArg(param.into_truncated()))?;

    let val_as_str = arg.value.as_str();

    let value: T = val_as_str.parse().map_err(|_| CliError::Parse(param.into_truncated()))?;

    Ok(value)
  }

  fn get_str_param<'a>(&'a self, param: &str) -> Option<&'a str> {
    self
      .iter()
      .find(|arg| arg.param.eq_ignore_ascii_case(param))
      .map(move |arg| arg.value.as_str())
  }

  fn contains_param(&self, str: &str) -> bool {
    self.iter().any(|arg| arg.param.contains(str))
  }
}
