use heapless::Vec;
use rp_pico::hal::gpio::{self, Function, Pin, PinId, PullType};

pub const MCU_PINS_NO: usize = 30;

pub type InputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioInput>, gpio::PullUp>;
pub type OutputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>;
pub type Raw<P> = gpio::Pin<P, gpio::FunctionNull, gpio::PullDown>;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Io Pins
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct IoPins<T, const N: usize> {
  locations: [Option<usize>; MCU_PINS_NO],
  pins:      Vec<T, N>,
}

impl<I: PinId, F: Function, P: PullType, const N: usize> IoPins<Pin<I, F, P>, N> {
  //
  /// Creates a new `IoPins` collection from a vector of pins.
  pub fn new(pins: Vec<Pin<I, F, P>, N>) -> Self {
    let mut locations = [None; MCU_PINS_NO];

    // Storing vec pos into a location array for retrieval
    for (index, pin) in pins.iter().enumerate() {
      let gpio_id = pin.id().num as usize;
      let slot = locations.get_mut(gpio_id).expect("Failed inserting pin id loc");
      *slot = Some(index);
    }

    Self { locations, pins }
  }
}

impl<T, const N: usize> IoPins<T, N> {
  /// Get a mutable reference to a pin by its GPIO number.
  #[inline]
  pub fn get_pin(&mut self, id: usize) -> Option<&mut T> {
    // pin lookup
    self.locations.get(id).and_then(|opt_pos| *opt_pos).and_then(|pos| self.pins.get_mut(pos))
  }
}
