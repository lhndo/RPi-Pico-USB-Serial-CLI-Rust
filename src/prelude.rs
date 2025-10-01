//! Prelude

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Prelude
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![allow(unused_imports)]
pub use core::str::FromStr;
pub use core::sync::atomic::Ordering;

pub use crate::adcs::{AdcConversion, TEMP_SENSE_CHN};
pub use crate::delay::DELAY;
pub use crate::device::*;
pub use crate::device::{Device, PinID, TimerExt};
pub use crate::fifo_buffer::{AsStr, FifoBuffer};
pub use crate::gpios::IoPins;
pub use crate::pwms::PwmChannelExt;
pub use crate::serial_io::SERIAL;
pub use crate::tasklet::Tasklet;
pub use crate::{print, println, with_pwm_slice};

pub use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
pub use embedded_hal::pwm::SetDutyCycle;
pub use embedded_hal_0_2::blocking::delay::{DelayMs, DelayUs};
pub use heapless::{String, Vec};

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, warn};
