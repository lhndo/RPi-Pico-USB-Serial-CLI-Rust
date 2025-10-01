//! Error implementation

pub use core::fmt;

pub use heapless::String;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub const ERR_STR_LENGTH: usize = 64;

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
        write!(f, "argument parse: {e}")
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
