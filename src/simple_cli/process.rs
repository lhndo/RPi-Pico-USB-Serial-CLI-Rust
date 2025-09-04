use super::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub const CLI_READ_BUFFER_LENGTH: usize = 128;
const MAX_CMD_CALL_LENGTH: usize = 24;
const MAX_NUMBER_PARAMS: usize = 5;
const MAX_PARAM_LENGTH: usize = 16;
const MAX_VALUE_LENGTH: usize = 64;
const ERR_STR_LENGTH: usize = 64;

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
      CliError::BufferWrite => write!(f, "failed to process the buffer into a valid command!"),
      CliError::IoInput => write!(f, "IO Input!"),
      CliError::Parse(e) => {
        write!(f, "argument parse, arg: {}", e.as_str())
      }
      CliError::MissingArg(e) => {
        write!(f, "missing argument <{}>", e.as_str())
      }
      CliError::CmdExec(e) => write!(f, "command failed with: {}", e.as_str()),
      CliError::CmdNotFound(e) => {
        write!(f, "command not found: {}", e.as_str())
      }
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
  commands: &'static [Command],
}

impl Cli {
  pub fn new(commands: &'static [Command]) -> Self {
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

    let cmd =
      self.commands.iter().find(|x| x.name.eq_ignore_ascii_case(cmd_name)).ok_or_else(|| {
        String::try_from(cmd_name).map(CliError::CmdNotFound).unwrap_or(CliError::BufferWrite)
      })?;

    // Execute Command
    (cmd.func)(&cmd_arg, device)?;
    Ok(())
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Input Commands
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Debug, Default)]
struct CommandWithArgs {
  cmd:  String<MAX_CMD_CALL_LENGTH>,
  args: Vec<Arguments, MAX_NUMBER_PARAMS>,
}

#[derive(Debug, Default, Clone)]
pub struct Arguments {
  param: String<MAX_PARAM_LENGTH>,
  value: String<MAX_VALUE_LENGTH>,
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

  // --- Stage 2: Split at witespace, split main command, split arguments ---
  let mut iter = processed_buf.split_ascii_whitespace();
  let cmd_str = iter.next().unwrap_or("help"); // defaulting to help command on error
  let command: String<MAX_CMD_CALL_LENGTH> =
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

// —————————————————————————————————— Get Parsed String Param —————————————————————————————————————

/// Matches a string parameter name and retrives the string from an argument list
pub fn get_str_param<'a>(param: &str, arg_list: &'a [Arguments]) -> Option<&'a str> {
  arg_list.iter().find(|arg| arg.param.eq_ignore_ascii_case(param)).map(|arg| arg.value.as_str())
}
