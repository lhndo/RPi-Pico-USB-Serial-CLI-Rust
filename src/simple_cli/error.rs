//! Error implementation

pub use heapless::String;
use thiserror::Error;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub const ERR_STR_LENGTH: usize = 48;

pub type Result<T> = core::result::Result<T, Error>;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Errors
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum Error {
  #[error("failed to generate buffer!")]
  BufferWrite,
  #[error("while parsing buffer!")]
  ParseBuffer,
  #[error("IO Input!")]
  IoInput,
  #[error("parsing arg: {0}")]
  Parse(String<ERR_STR_LENGTH>),
  #[error("missing arg <{0}>")]
  MissingArg(String<ERR_STR_LENGTH>),
  #[error("command failed with: {0}")]
  CmdExec(String<ERR_STR_LENGTH>),
  #[error("command not found: {0}")]
  CmdNotFound(String<ERR_STR_LENGTH>),
  #[error("command too long!")]
  CommandTooLong,
  #[error("argument too long!")]
  ArgTooLong,
  #[error("too many arguments!")]
  TooManyArgs,
  #[error("critical failure!")]
  CriticalFail,
  #[error("exited!")]
  Exit,
  #[error(transparent)]
  Configuration(#[from] crate::config::Error),
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
