//! Prelude

#![allow(unused_imports)]

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

// sio fifo events for multi-core
pub const E_WAKE_UP: u32 = 1;
pub const E_DONE: u32 = 0;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Prelude
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub use core::str::FromStr;
pub use core::sync::atomic::Ordering;

pub use crate::main_core1::{CORE1_QUEUE, Event};
pub use crate::system::adcs::{AdcConversion, TEMP_SENSE_CHN};
pub use crate::system::config::CONFIG;
pub use crate::system::config::Error as ConfigError;
pub use crate::system::delay::DELAY;
pub use crate::system::device::*;
pub use crate::system::device::{Device, TimerExt};
pub use crate::system::gpios::{InputType, IoPins, OutputType};
pub use crate::system::pwms::PwmChannelExt;
pub use crate::system::serial_io::SERIAL;
pub use crate::utils::fifo_buffer::{AsStr, FifoBuffer};
pub use crate::utils::log::{LOG, LogLevel};
pub use crate::utils::tasklet::Tasklet;

pub use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
pub use embedded_hal::pwm::SetDutyCycle;
pub use embedded_hal_0_2::blocking::delay::{DelayMs, DelayUs};
pub use heapless::{String, Vec};

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                               Log
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub use crate::{gpio, print, println, with_pwm_slice};

// Logging
#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, trace, warn};

#[cfg(not(feature = "defmt"))]
pub use crate::{debug, error, info, trace, warn};
