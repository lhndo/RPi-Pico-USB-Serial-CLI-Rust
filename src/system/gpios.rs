//! Input/Output GP Pin Storage for the RP2040 microcontroller

use super::config::Error;
use super::config::Result;

use hal::gpio::{self, Function, Pin, PullType};
use rp2040_hal::{self as hal};

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub const NUM_MCU_PINS: usize = 30;

pub type InputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioInput>, gpio::PullUp>;
pub type OutputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Io Pins
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct IoPins<T> {
  pins: [Option<T>; NUM_MCU_PINS],
}

impl<F: Function, P: PullType> IoPins<Pin<gpio::DynPinId, F, P>> {
  //
  /// Creates a new `IoPins` collection from a vector of pins.
  pub fn new() -> Self {
    let pins: [Option<Pin<gpio::DynPinId, F, P>>; NUM_MCU_PINS] = core::array::from_fn(|_| None);

    Self { pins }
  }

  /// Register pin
  pub fn register(&mut self, pin: Pin<gpio::DynPinId, F, P>) {
    let id = pin.id().num;
    if id >= NUM_MCU_PINS as u8 {
      panic!("ID > NUM_MCU_PINS")
    }
    self.pins[id as usize] = Some(pin);
  }
}

impl<T> IoPins<T> {
  /// Get a mutable reference to a pin by its GPIO number.
  #[inline]
  pub fn get(&mut self, id: u8) -> Result<&mut T> {
    if id >= NUM_MCU_PINS as u8 {
      return Err(Error::OutOfBounds);
    }

    self.pins[id as usize].as_mut().ok_or(Error::GpioNotFound)
  }
}
