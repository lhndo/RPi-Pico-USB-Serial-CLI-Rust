//! Input/Output GP Pin Storage for the RP2040 microcontroller

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

  pub fn add_pin(&mut self, pin: Pin<gpio::DynPinId, F, P>, id: u8) {
    if id >= NUM_MCU_PINS as u8 {
      panic!("ID > NUM_MCU_PINS")
    }
    self.pins[id as usize] = Some(pin);
  }

  /// Add a pin by GPIO ID
  ///
  /// # Safety
  /// The caller must ensure that no other instance of this pin exists
  pub fn register_pin_by_gpio_id(&mut self, gpio_id: u8) {
    if gpio_id >= NUM_MCU_PINS as u8 {
      panic!("ID > NUM_MCU_PINS")
    }

    let pin = unsafe {
      gpio::new_pin(gpio::DynPinId {
        bank: gpio::DynBankId::Bank0,
        num:  gpio_id,
      })
    };

    let pin = pin
      .try_into_function::<F>()
      .map(|p| p.into_pull_type::<P>())
      .expect("gpio pin config error");

    self.pins[gpio_id as usize] = Some(pin);
  }
}

impl<T> IoPins<T> {
  /// Get a mutable reference to a pin by its GPIO number.
  #[inline]
  pub fn get_by_gpio_id(&mut self, id: u8) -> Option<&mut T> {
    if id >= NUM_MCU_PINS as u8 {
      return None;
    }

    self.pins[id as usize].as_mut()
  }
}
