//! Global Delay Provider
//! Safe, non-efficient spinning non-interruptible.
//! Reach for device.timer.delay_ms() in most use cases

use core::cell::RefCell;
use cortex_m::delay::Delay as CortexmDelay;
use critical_section::{Mutex, with as free};

// ———————————————————————————————————————————————————————————————————————————————————————
//                                        Globals
// ———————————————————————————————————————————————————————————————————————————————————————

/// The global instance of the `DelayHandle`.
pub const DELAY: DelayHandle = DelayHandle;

// Use a Mutex to safely wrap the RefCell, making it thread-safe (Sync).
// The Option handles the uninitialized vs. initialized state.
// Storing the delay object from init
static DELAY_CELL: Mutex<RefCell<Option<CortexmDelay>>> = Mutex::new(RefCell::new(None));

// ———————————————————————————————————————————————————————————————————————————————————————
//                                         Init
// ———————————————————————————————————————————————————————————————————————————————————————

/// Initialise the DELAY global object once
pub fn init(delay: CortexmDelay) {
  // Panic if Some, initialize if None
  free(|cs| {
    let mut cell = DELAY_CELL.borrow_ref_mut(cs);
    if cell.is_some() {
      panic!("DELAY already initialized");
    }

    cell.replace(delay);
  });
}

// ———————————————————————————————————————————————————————————————————————————————————————
//                                      DelayHandle
// ———————————————————————————————————————————————————————————————————————————————————————

/// A zero-cost global delay handler for a basic spinning delay
pub struct DelayHandle;

impl DelayHandle {
  /// Executes a closure with a mutable reference to the delay object.
  /// Panics if the DELAY has not been initialized.
  fn with<F>(&self, f: F)
  where
    F: FnOnce(&mut CortexmDelay),
  {
    free(|cs| {
      if let Some(cell) = DELAY_CELL.borrow_ref_mut(cs).as_mut() {
        f(cell);
      }
      else {
        panic!("DELAY not initialized");
      }
    });
  }

  /// Pauses execution for a minimum number of milliseconds.
  pub fn ms(&self, ms: u32) {
    self.with(|delay| delay.delay_ms(ms));
  }

  /// Pauses execution for a minimum number of microseconds.
  pub fn us(&self, us: u32) {
    self.with(|delay| delay.delay_us(us));
  }
}
