//! Error implementation

pub use core::fmt;

pub use heapless::String;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub const ERR_STR_LENGTH: usize = 48;

pub type Result<T> = core::result::Result<T, CliError>;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Errors
// ————————————————————————————————————————————————————————————————————————————————————————————————

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
  CommandTooLong,
  ArgTooLong,
  TooManyArgs,
  CriticalFail,
  Other,
  Exit,
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", match self {
      CliError::BufferWrite => "failed to generate buffer!",
      CliError::ParseBuffer => "while parsing buffer!",
      CliError::IoInput => "IO Input!",
      CliError::Parse(e) => return write!(f, "argument parse: {e}"),
      CliError::MissingArg(e) => return write!(f, "missing argument <{e}>"),
      CliError::CmdExec(e) => return write!(f, "command failed with: {e}"),
      CliError::CmdNotFound(e) => return write!(f, "command not found: {e}"),
      CliError::CommandTooLong => "command too long!",
      CliError::ArgTooLong => "argument too long!",
      CliError::TooManyArgs => "too many arguments!",
      CliError::CriticalFail => "critical failure!",
      CliError::Exit => "exit!",
      CliError::Other => "internal error!",
    })
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

// ———————————————————————————————————————— Into Truncate ——————————————————————————————————————————

/// Converts from &str to heapless String<N> truncating the length to N
pub trait IntoTruncate {
  fn into_truncated<const N: usize>(self) -> String<N>;
}

impl IntoTruncate for &str {
  fn into_truncated<const N: usize>(self) -> String<N> {
    let mut s = String::<N>::new();

    let end = if self.len() <= N {
      self.len()
    }
    else {
      let mut end = N;
      while !self.is_char_boundary(end) {
        end -= 1;
      }
      end
    };

    let _ = s.push_str(&self[..end]);
    s
  }
}
