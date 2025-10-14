//! Prelude

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Prelude
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![allow(unused_imports)]
pub use core::str::FromStr;
pub use core::sync::atomic::Ordering;

pub use crate::adcs::{AdcConversion, TEMP_SENSE_CHN};
pub use crate::config::CONFIG;
pub use crate::config::Error as ConfigError;
pub use crate::core1::{CORE1_QUEUE, Event};
pub use crate::delay::DELAY;
pub use crate::device::*;
pub use crate::device::{Device, TimerExt};
pub use crate::fifo_buffer::{AsStr, FifoBuffer};
pub use crate::gpios::{InputType, IoPins, OutputType};
pub use crate::log::{LOG, LogLevel};
pub use crate::pwms::PwmChannelExt;
pub use crate::serial_io::SERIAL;
pub use crate::tasklet::Tasklet;

pub use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
pub use embedded_hal::pwm::SetDutyCycle;
pub use embedded_hal_0_2::blocking::delay::{DelayMs, DelayUs};
pub use heapless::{String, Vec};

pub use crate::{gpio, print, println, with_pwm_slice};

// Logging
#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, trace, warn};

#[cfg(not(feature = "defmt"))]
pub use crate::{debug, error, info, trace, warn};

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

// sio fifo events for multi-core
pub const E_WAKE_UP: u32 = 1;
pub const E_DONE: u32 = 0;
