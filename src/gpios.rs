//! Input/Output GP Pin Storage for the RP2040 microcontroller

use rp_pico::hal::gpio::{self, Function, Pin, PinId, PullType};

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

impl<I: PinId, F: Function, P: PullType> IoPins<Pin<I, F, P>> {
  //
  /// Creates a new `IoPins` collection from a vector of pins.
  pub fn new() -> Self {
    let pins: [Option<Pin<I, F, P>>; NUM_MCU_PINS] = core::array::from_fn(|_| None);

    Self { pins }
  }

  pub fn add_pin(&mut self, pin: Pin<I, F, P>, id: u8) {
    if id >= NUM_MCU_PINS as u8 {
      panic!("ID > NUM_MCU_PINS")
    }
    self.pins[id as usize] = Some(pin);
  }
}

impl<T> IoPins<T> {
  /// Get a mutable reference to a pin by its GPIO number.
  #[inline]
  pub fn get_by_id(&mut self, id: u8) -> Option<&mut T> {
    if id >= NUM_MCU_PINS as u8 {
      return None;
    }

    self.pins[id as usize].as_mut()
  }
}
