//! Simple CLI

pub mod commands;
pub mod program;

use crate::prelude::*;
use heapless::{String, Vec};

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Errors
// ————————————————————————————————————————————————————————————————————————————————————————————————

type Result<T> = core::result::Result<T, CliError>;

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CliError {
  BufferWrite,
  IoInput,
  Parse(String<ERR_STR_LENGTH>),
  MissingArg(String<ERR_STR_LENGTH>),
  CmdExec(String<ERR_STR_LENGTH>),
  CmdNotFound(String<ERR_STR_LENGTH>),
  CriticalFail,
  Other,
  Exit,
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CliError::BufferWrite => {
        write!(f, "failed to process the buffer into a valid command!")
      },
      CliError::IoInput => write!(f, "IO Input!"),
      CliError::Parse(e) => {
        write!(f, "argument parse, arg: {}", e.as_str())
      },
      CliError::MissingArg(e) => {
        write!(f, "missing argument <{}>", e.as_str())
      },
      CliError::CmdExec(e) => write!(f, "command failed with: {}", e.as_str()),
      CliError::CmdNotFound(e) => {
        write!(f, "command not found: {}", e.as_str())
      },
      CliError::CriticalFail => write!(f, "critical failure!"),
      CliError::Exit => write!(f, "exit!"),
      CliError::Other => write!(f, "internal error!"),
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Cli {
  commands: CommandList,
}

impl Cli {
  pub fn new(commands: CommandList) -> Self {
    Self { commands }
  }

  pub fn execute(&mut self, input: &str, device: &mut Device) -> Result<()> {
    let command = split_into_cmd_args(input)?;
    self.execute_command(command, device)?;
    Ok(())
  }

  fn execute_command(&mut self, command: CommandWithArgs, device: &mut Device) -> Result<()> {
    let cmd_name = command.cmd.as_str();
    let cmd_arg = command.args;

    // Check if built-in help was called
    if cmd_name == "help" {
      self.built_in_help();
      return Ok(());
    }

    let cmd = self.commands.get_command(cmd_name)?;
    // Execute Command
    (cmd.func)(&cmd_arg, device)
  }

  pub fn built_in_help(&self) {
    println!("\nAvailable Commands:");
    println!("-----------------------------");

    for command in self.commands.commands.iter() {
      println!("{} - {}", command.name, command.desc);
    }
    println!("-----------------------------");
    println!("For more information type: command_name help\n");
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                          Command Structs
// —————————————————————————————————————————————————————————————————————————————————————————————————

const MAX_CMDS: usize = 20;
type FunctionCmd = fn(&[Arguments], &mut Device) -> Result<()>;

// ———————————————————————————————————————— Command List ———————————————————————————————————————————

#[derive(Default, Debug)]
pub struct CommandList {
  pub commands: Vec<Command, MAX_CMDS>,
}

impl CommandList {
  pub fn register_command(&mut self, command: Command) {
    let _ = self.commands.push(command);
  }

  pub fn get_command(&self, name: &str) -> Result<&Command> {
    if let Some(cmd) = self.commands.iter().find(|cmd| cmd.name.eq_ignore_ascii_case(name)) {
      Ok(cmd)
    }
    else {
      Err(
        String::try_from(name)
          .map(CliError::CmdNotFound)
          .unwrap_or(CliError::BufferWrite),
      )
    }
  }

  pub fn get_description(&self, cmd_name: &str) -> Result<&'static str> {
    let command = self.get_command(cmd_name)?;
    Ok(command.desc)
  }
}

// ——————————————————————————————————————————— Command —————————————————————————————————————————————

#[derive(Debug)]
pub struct Command {
  pub name: &'static str,
  pub desc: &'static str,
  pub func: FunctionCmd,
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Inputs
// ————————————————————————————————————————————————————————————————————————————————————————————————

const CLI_READ_BUFFER_LENGTH: usize = 128;
const MAX_CMD_LENGTH: usize = 24;
const MAX_NUMBER_PARAMS: usize = 5;
const MAX_PARAM_LENGTH: usize = 16;
const MAX_VALUE_LENGTH: usize = 64;
const ERR_STR_LENGTH: usize = 64;

#[derive(Debug, Default)]
struct CommandWithArgs {
  cmd:  String<MAX_CMD_LENGTH>,
  args: Vec<Arguments, MAX_NUMBER_PARAMS>,
}

#[derive(Debug, Default, Clone)]
pub struct Arguments {
  pub param: String<MAX_PARAM_LENGTH>,
  pub value: String<MAX_VALUE_LENGTH>,
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Helper Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————

// —————————————————————————————————— Split Into Cmd and Args —————————————————————————————————————

/// Takes an input string and processes it creating a CommandWithArgs struct
fn split_into_cmd_args(input: &str) -> Result<CommandWithArgs> {
  // --- Stage 1: All text to lowercase, detect and strip quotes
  // and switch spaces inside with a key (0x1E) symbol ---
  let mut processed_buf: String<CLI_READ_BUFFER_LENGTH> = String::new();
  let mut in_quotes = false;
  for char in input.chars() {
    match char {
      '"' => {
        in_quotes = !in_quotes;
      },
      ' ' if in_quotes => {
        processed_buf.push(char::from(0x1E)).map_err(|_| CliError::BufferWrite)?;
      },
      c if c.is_ascii_uppercase() => {
        processed_buf.push(c.to_ascii_lowercase()).map_err(|_| CliError::BufferWrite)?;
      },
      c => {
        processed_buf.push(c).map_err(|_| CliError::BufferWrite)?;
      },
    }
  }

  // --- Stage 2: Split at witespace, split main command, split arguments ---
  let mut iter = processed_buf.split_ascii_whitespace();
  let cmd_str = iter.next().unwrap_or("help"); // defaulting to help command on error
  let command: String<MAX_CMD_LENGTH> =
    String::try_from(cmd_str).map_err(|_| CliError::BufferWrite)?;

  let mut split_input = CommandWithArgs {
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
        .push(Arguments {
          param: arg_param,
          value: String::new(),
        })
        .map_err(|_| CliError::BufferWrite)?;
    }
    else if len >= 2 {
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
        .push(Arguments {
          param: arg_param,
          value: arg_value,
        })
        .map_err(|_| CliError::BufferWrite)?;
    }
  }
  Ok(split_input)
}

// —————————————————————————————————————— Get Parsed Param ————————————————————————————————————————

/// Matches a string parameter name and retrives the value from an argument list
pub fn get_parsed_param<T>(param: &str, arg_list: &[Arguments]) -> Result<T>
where
  T: FromStr,
{
  // Find argument
  let arg = arg_list.iter().find(|s| s.param.eq_ignore_ascii_case(param)).ok_or_else(|| {
    String::try_from(param)
      .map(CliError::MissingArg)
      .unwrap_or(CliError::BufferWrite)
    // Or map directly if try_from returns a compatible error
  })?;

  let val_as_str = arg.value.as_str();

  let value: T = val_as_str
    .parse()
    .map_err(|_| String::try_from(param).map(CliError::Parse).unwrap_or(CliError::BufferWrite))?;

  Ok(value)
}

// —————————————————————————————————— Get Parsed String Param —————————————————————————————————————

/// Matches a string parameter name and retrives the string from an argument list
pub fn get_str_param<'a>(param: &str, arg_list: &'a [Arguments]) -> Option<&'a str> {
  arg_list
    .iter()
    .find(|arg| arg.param.eq_ignore_ascii_case(param))
    .map(|arg| arg.value.as_str())
}

// —————————————————————————————————————— Contains argument ————————————————————————————————————————

/// Checks if the argument list contains a certain str param and returns true or false
pub fn contains_param(str: &str, args: &[Arguments]) -> bool {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains(str)) {
    return true;
  }
  false
}
