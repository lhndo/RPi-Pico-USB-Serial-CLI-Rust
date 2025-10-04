//! Basic logging interface for use when "defmt" features are disabled
//! Logging is printed in the CLI only when a CLI connection is available
//!
//! Example:
//! ```rust
//! LOG.set(LogLevel::Trace);
//! info!("This is an info msg");
//! ```

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;

use core::fmt;

pub static LOG: Log = Log { level: AtomicU8::new(5) }; // Defaults to Trace

pub struct Log {
  level: AtomicU8,
}

impl Log {
  pub fn get(&self) -> LogLevel {
    let level = self.level.load(Ordering::Relaxed);
    level.into()
  }

  pub fn set(&self, level: LogLevel) {
    let level = level.into();
    self.level.store(level, Ordering::Relaxed);
  }

  pub fn get_as_u8(&self) -> u8 {
    self.level.load(Ordering::Relaxed)
  }
}

#[derive(Debug)]
#[repr(u8)]
pub enum LogLevel {
  Off,   // 0
  Error, // 1
  Warn,  // 2
  Info,  // 3
  Debug, // 4
  Trace, // 5
}

impl From<LogLevel> for u8 {
  fn from(level: LogLevel) -> Self {
    level as u8
  }
}

impl From<u8> for LogLevel {
  fn from(level: u8) -> Self {
    match level {
      0 => LogLevel::Off,
      1 => LogLevel::Error,
      2 => LogLevel::Warn,
      3 => LogLevel::Info,
      4 => LogLevel::Debug,
      _ => LogLevel::Trace,
    }
  }
}

impl fmt::Display for LogLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", match self {
      LogLevel::Off => "[OFF]",
      LogLevel::Error => "[ERROR]",
      LogLevel::Warn => "[WARN]",
      LogLevel::Info => "[INFO]",
      LogLevel::Debug => "[DEBUG]",
      LogLevel::Trace => "[TRACE]",
    })
  }
}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
      if $crate::log::LOG.get_as_u8() >= 1 {
        $crate::print!("[ERROR] ");
        $crate::println!($($arg)*);
      }
}}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
      if $crate::log::LOG.get_as_u8() >= 2 {
        $crate::print!("[WARN ] ");
        $crate::println!($($arg)*);
      }
}}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
      if $crate::log::LOG.get_as_u8() >= 3 {
        $crate::print!("[INFO ] ");
        $crate::println!($($arg)*);
      }
}}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
      if $crate::log::LOG.get_as_u8() >= 4 {
        $crate::print!("[DEBUG] ");
        $crate::println!($($arg)*);
      }
}}

#[cfg(not(feature = "defmt"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
      if $crate::log::LOG.get_as_u8() >= 5 {
        $crate::print!("[TRACE] ");
        $crate::println!($($arg)*);
      }
}}
