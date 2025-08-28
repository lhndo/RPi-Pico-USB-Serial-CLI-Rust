use core::panic;

use rp_pico::hal::gpio::{self, Function, Pin, PinId, PullType};

pub const NUM_MCU_PINS: usize = 30;
pub const NUM_MAX_DEF_PINS: usize = 15; // max number of input or output pins stored in the device

pub type InputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioInput>, gpio::PullUp>;
pub type OutputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>;
pub type Raw<P> = gpio::Pin<P, gpio::FunctionNull, gpio::PullDown>;

const EMPTY_SLOT: usize = 255;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Io Pins
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct IoPins<T> {
  locations: [usize; NUM_MCU_PINS],
  pins:      [Option<T>; NUM_MAX_DEF_PINS],
}

impl<I: PinId, F: Function, P: PullType> IoPins<Pin<I, F, P>> {
  //
  /// Creates a new `IoPins` collection from a vector of pins.
  pub fn new<const N: usize>(pins: [Pin<I, F, P>; N]) -> Self {
    let mut locations = [EMPTY_SLOT; NUM_MCU_PINS];
    let mut storage: [Option<Pin<I, F, P>>; NUM_MAX_DEF_PINS] = core::array::from_fn(|_| None);

    // Storing vec pos into a location array for retrieval
    for (index, pin) in pins.into_iter().enumerate() {
      let gpio_id = pin.id().num as usize;
      if gpio_id < NUM_MCU_PINS {
        locations[gpio_id] = index;
      } else {
        panic!("pin out of bounds")
      }
      // Move the pin into the `storage` array.
      storage[index] = Some(pin);
    }

    Self { locations, pins: storage }
  }
}

impl<T> IoPins<T> {
  /// Get a mutable reference to a pin by its GPIO number.
  #[inline]
  pub fn get_pin(&mut self, id: usize) -> Option<&mut T> {
    if id >= NUM_MCU_PINS || self.locations[id] == EMPTY_SLOT {
      return None;
    }
    let index = self.locations[id];
    self.pins[index].as_mut()
  }
}
