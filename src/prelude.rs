// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Prelude
// ————————————————————————————————————————————————————————————————————————————————————————————————


#![allow(unused_imports)]
pub use core::fmt::Write;

pub use crate::delay::DELAY;
pub use crate::device::*;
pub use crate::device::{AdcTools, Device, TimerExt};
pub use crate::fifo_buffer::{AsStr, FifoBuffer};
pub use crate::serial_io::SERIAL;
pub use crate::{print, println};

pub use cortex_m::prelude::*;
pub use embedded_hal::digital::{OutputPin, StatefulOutputPin};
pub use heapless::{String, Vec};
