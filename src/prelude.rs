// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Prelude
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![allow(unused_imports)]
pub use core::fmt::{self, Write};
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

pub use cortex_m::prelude::*;
pub use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
pub use embedded_hal::pwm::SetDutyCycle;
pub use heapless::{String, Vec};
pub use rp_pico::hal::pwm;

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, warn};
