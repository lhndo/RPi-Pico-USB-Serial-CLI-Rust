// -----------------------------------------------------------------------------
//                                 SIMPLE CLI
// -----------------------------------------------------------------------------
pub mod cli_commands;

use crate::prelude::{print, println};

use core::fmt::{self, Write};
use core::str::FromStr;
use heapless::{String, Vec};


// -----------------------------------------------------------------------------
//                                   Globals
// -----------------------------------------------------------------------------


pub const CLI_READ_BUFFER_LENGTH: usize = 128;
const MAX_CMD_CALL_LENGTH: usize = 24;
const MAX_PARAM_LENGTH: usize = 16;
const MAX_VALUE_LENGTH: usize = 64;
const MAX_NUMBER_PARAMS: usize = 5;
const ERR_STR_LENGTH: usize = 64;


type Result<T> = core::result::Result<T, CliError>;
// -----------------------------------------------------------------------------
//                                   Errors
// -----------------------------------------------------------------------------

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CliError {
  BufferWrite,
  IoInput,
  Parse(String<ERR_STR_LENGTH>),
  MissingArg(String<ERR_STR_LENGTH>),
  CmdExec,
  CmdNotFound(String<ERR_STR_LENGTH>),
  CriticalFail,
  Other,
  Exit,
}

#[allow(unreachable_patterns)]
impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CliError::BufferWrite => write!(f, "buffer write"),
      CliError::IoInput => write!(f, "IO Input"),
      CliError::Parse(e) => {
        write!(f, "argument parse, arg: {}", e.as_str())
      }
      CliError::MissingArg(e) => {
        write!(f, "missing argument <{}>", e.as_str())
      }
      CliError::CmdExec => write!(f, "command failed execution"),
      CliError::CmdNotFound(e) => {
        write!(f, "command not found: {}", e.as_str())
      }
      CliError::CriticalFail => write!(f, "critical failure"),
      CliError::Exit => write!(f, "exit"),
      CliError::Other => write!(f, "internal error"),
      _ => write!(f, "unexpected error"),
    }
  }
}

// -----------------------------------------------------------------------------
//                              Command Struct
// -----------------------------------------------------------------------------

pub struct Command {
  pub name: &'static str,
  pub desc: &'static str,
  pub func: fn(&[Arg]) -> Result<()>,
}

#[derive(Debug, Default)]
struct CmdWithArgs {
  cmd:  String<MAX_CMD_CALL_LENGTH>,
  args: Vec<Arg, MAX_NUMBER_PARAMS>,
}

#[derive(Debug, Default, Clone)]
pub struct Arg {
  param: String<MAX_PARAM_LENGTH>,
  value: String<MAX_VALUE_LENGTH>,
}

// -----------------------------------------------------------------------------
//                                    Cli
// -----------------------------------------------------------------------------

pub struct Cli {
  commands: &'static [Command],
}

// -------------------------------- Cli Impl -----------------------------------

impl Cli {
  pub fn new(commands: &'static [Command]) -> Self {
    Self { commands }
  }

  pub fn init(&mut self) {
    println!("\nSimple CLI - type 'help' for commands");
  }

  // --------------------------------- Run -----------------------------------

  pub fn run(&mut self) -> Result<()> {
    println!("\nEnter Command >> ");

    // CLI Input
    // let input = self.io_reader.read_line();
    let input = "self.io_reader.read_line()"; // TODO: Port to Program

    println!("--------------------");

    // Quick System Commands
    if input.eq_ignore_ascii_case("exit") {
      return Err(CliError::Exit);
    }

    // CLI Execution
    self.execute(input)?;

    println!("--------------------");

    Ok(())
  }

  // ------------------------------- Execute ---------------------------------

  pub fn execute(&mut self, input: &str) -> Result<()> {
    let command = split_into_cmd_args(input)?;
    self.execute_cmd(&command)?;
    Ok(())
  }

  fn execute_cmd(&mut self, in_cmd: &CmdWithArgs) -> Result<()> {
    let cmd_name = in_cmd.cmd.as_str();
    let cmd_arg = &in_cmd.args;

    let cmd =
      self.commands.iter().find(|x| x.name.eq_ignore_ascii_case(cmd_name)).ok_or_else(|| {
        String::try_from(cmd_name).map(CliError::CmdNotFound).unwrap_or(CliError::BufferWrite)
      })?;

    // Executing command
    (cmd.func)(cmd_arg)?;
    Ok(())
  }
}

// -----------------------------------------------------------------------------
//                              Helper Functions
// -----------------------------------------------------------------------------

// ---------------------- Split into command and arguments -------------------------

fn split_into_cmd_args(input: &str) -> Result<CmdWithArgs> {
  // --- Stage 1: Process quotes and case, building intermediate String ---
  let mut processed_buf: String<CLI_READ_BUFFER_LENGTH> = String::new();
  let mut in_quotes = false;
  for char in input.chars() {
    match char {
      '"' => {
        in_quotes = !in_quotes;
      }
      ' ' if in_quotes => {
        processed_buf.push(char::from(0x1E)).map_err(|_| CliError::BufferWrite)?;
      }
      c if c.is_ascii_uppercase() => {
        processed_buf.push(c.to_ascii_lowercase()).map_err(|_| CliError::BufferWrite)?;
      }
      c => {
        processed_buf.push(c).map_err(|_| CliError::BufferWrite)?;
      }
    }
  }

  // --- Stage 2: Split and parse command/args ---
  let mut iter = processed_buf.split_ascii_whitespace();
  let cmd_str = iter.next().unwrap_or("help");
  let command: String<MAX_CMD_CALL_LENGTH> =
    String::try_from(cmd_str).map_err(|_| CliError::BufferWrite)?;

  let mut split_input = CmdWithArgs {
    cmd:  command,
    args: Vec::new(),
  };

  // --- Stage 3: Process arguments using original structure ---
  for word in iter {
    if word.is_empty() || word == "=" {
      continue;
    }

    let elements: heapless::Vec<&str, 2> = word.split('=').collect();

    let len = elements.len();

    if len == 1 {
      let arg_param = String::try_from(elements[0]).map_err(|_| CliError::BufferWrite)?;
      split_input
        .args
        .push(Arg {
          param: arg_param,
          value: String::new(),
        })
        .map_err(|_| CliError::BufferWrite)?;
    } else if len >= 2 {
      let arg_param = String::try_from(elements[0]).map_err(|_| CliError::BufferWrite)?;

      let mut arg_value: String<MAX_VALUE_LENGTH> = String::new();

      // Restoring space characters
      for char in elements[1].chars() {
        let c_to_push = if char == char::from(0x1E) { ' ' } else { char };
        arg_value.push(c_to_push).map_err(|_| CliError::BufferWrite)?;
      }

      // Creating argument with value
      split_input
        .args
        .push(Arg {
          param: arg_param,
          value: arg_value,
        })
        .map_err(|_| CliError::BufferWrite)?;
    }
  }
  Ok(split_input)
}

// ---------------------------- Get Parsed Param -------------------------------

pub fn get_parsed_param<T>(param: &str, arg_list: &[Arg]) -> Result<T>
where T: FromStr {
  // Find argument
  let arg = arg_list.iter().find(|s| s.param.eq_ignore_ascii_case(param)).ok_or_else(|| {
    String::try_from(param).map(CliError::MissingArg).unwrap_or(CliError::BufferWrite)
    // Or map directly if try_from returns a compatible error
  })?;

  let val_as_str = arg.value.as_str();

  let value: T = val_as_str
    .parse()
    .map_err(|_| String::try_from(param).map(CliError::Parse).unwrap_or(CliError::BufferWrite))?;

  Ok(value)
}

// ------------------------------ Get Str Param --------------------------------

pub fn get_str_param<'a>(param: &str, arg_list: &'a [Arg]) -> Option<&'a str> {
  arg_list.iter().find(|arg| arg.param.eq_ignore_ascii_case(param)).map(|arg| arg.value.as_str())
}
