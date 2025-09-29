//! A Simple CLI Module

pub mod commands;
pub mod program;

pub use crate::prelude::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Errors
// ————————————————————————————————————————————————————————————————————————————————————————————————

const CLI_READ_BUFFER_LENGTH: usize = 128;
const ERR_STR_LENGTH: usize = 64;

const MAX_CMDS: usize = 20;
const MAX_CMD_LENGTH: usize = 24;

const MAX_NUMBER_PARAMS: usize = 5;
const MAX_PARAM_LENGTH: usize = 16;
const MAX_VALUE_LENGTH: usize = 64;

type Result<T> = core::result::Result<T, CliError>;

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CliError {
  BufferWrite,
  ParseBuffer,
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
      CliError::ParseBuffer => {
        write!(f, "while parsing buffer!")
      },
      CliError::IoInput => write!(f, "IO Input!"),
      CliError::Parse(e) => {
        write!(f, "argument parse, arg: {e}")
      },
      CliError::MissingArg(e) => {
        write!(f, "missing argument <{e}>")
      },
      CliError::CmdExec(e) => write!(f, "command failed with: {e}"),
      CliError::CmdNotFound(e) => {
        write!(f, "command not found: {e}")
      },
      CliError::CriticalFail => write!(f, "critical failure!"),
      CliError::Exit => write!(f, "exit!"),
      CliError::Other => write!(f, "internal error!"),
    }
  }
}

trait IntoTruncate {
  fn into_truncate<const N: usize>(self) -> String<N>;
}

impl IntoTruncate for &str {
  fn into_truncate<const N: usize>(self) -> String<N> {
    let mut s = String::<N>::new();
    let _ = s.push_str(self); // truncates if too long
    s
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
    let command = parse_input(input)?;
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
    (cmd.func)(cmd, &cmd_arg, device)
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

type FunctionCmd = fn(cmd: &Command, &[Arguments], &mut Device) -> Result<()>;

#[derive(Debug)]
pub struct Command {
  pub name: &'static str,
  pub desc: &'static str,
  pub help: &'static str,
  pub func: FunctionCmd,
}

impl Command {
  pub fn print_help(&self) {
    println!("{}", self.desc);
    println!("{}", self.help)
  }

  pub fn print_description(&self) {
    println!("{}", self.desc);
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Inputs
// ————————————————————————————————————————————————————————————————————————————————————————————————

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

// ————————————————————————————————————————— Parse Input ———————————————————————————————————————————

/// Takes an input string and processes it creating a CommandWithArgs struct
fn parse_input(input: &str) -> Result<CommandWithArgs> {
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
        processed_buf.push(char::from(0x1E)).map_err(|_| CliError::ParseBuffer)?;
      },
      c if c.is_ascii_uppercase() => {
        processed_buf.push(c.to_ascii_lowercase()).map_err(|_| CliError::ParseBuffer)?;
      },
      c => {
        processed_buf.push(c).map_err(|_| CliError::ParseBuffer)?;
      },
    }
  }

  // --- Stage 2: Split at witespace, split main command, split arguments ---
  let mut iter = processed_buf.split_ascii_whitespace();
  let cmd_str = iter.next().unwrap_or("help"); // defaulting to help command on error
  let command: String<MAX_CMD_LENGTH> =
    String::try_from(cmd_str).map_err(|_| CliError::ParseBuffer)?;

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
      let arg_param = String::try_from(elements[0]).map_err(|_| CliError::ParseBuffer)?;
      split_input
        .args
        .push(Arguments {
          param: arg_param,
          value: String::new(),
        })
        .map_err(|_| CliError::BufferWrite)?;
    }
    else if len >= 2 {
      let arg_param = String::try_from(elements[0]).map_err(|_| CliError::ParseBuffer)?;

      let mut arg_value: String<MAX_VALUE_LENGTH> = String::new();

      // Restoring space characters
      for char in elements[1].chars() {
        let c_to_push = if char == char::from(0x1E) { ' ' } else { char };
        arg_value.push(c_to_push).map_err(|_| CliError::ParseBuffer)?;
      }

      // Creating argument with value
      split_input
        .args
        .push(Arguments {
          param: arg_param,
          value: arg_value,
        })
        .map_err(|_| CliError::ParseBuffer)?;
    }
  }
  Ok(split_input)
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

impl ArgList for &[Arguments] {
  fn get_parsed_param<T>(&self, param: &str) -> Result<T>
  where
    T: FromStr,
  {
    let arg = self
      .iter()
      .find(|s| s.param.eq_ignore_ascii_case(param))
      .ok_or_else(|| CliError::MissingArg(param.into_truncate()))?;

    let val_as_str = arg.value.as_str();

    let value: T = val_as_str.parse().map_err(|_| CliError::Parse(param.into_truncate()))?;

    Ok(value)
  }

  fn get_str_param<'a>(&'a self, param: &str) -> Option<&'a str> {
    self
      .iter()
      .find(|arg| arg.param.eq_ignore_ascii_case(param))
      .map(move |arg| arg.value.as_str())
  }

  fn contains_param(&self, str: &str) -> bool {
    if let Some(arg) = self.iter().find(|arg| arg.param.contains(str)) {
      return true;
    }
    false
  }
}
