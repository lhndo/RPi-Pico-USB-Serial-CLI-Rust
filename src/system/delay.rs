//! Global Delay provider
//!
//! Single-threaded only: must not be called from interrupts.
//! Unsafe `Sync` implementation is used to avoid a critical-section Mutex, which would block interrupts.  
//! Alternatively use `device.timer.delay_ms()` .

use core::cell::RefCell;
use cortex_m::delay::Delay as CortexmDelay;

// ———————————————————————————————————————————————————————————————————————————————————————
//                                        Globals
// ———————————————————————————————————————————————————————————————————————————————————————

// #[thread_local] - required for multiple threads
pub static DELAY: DelayHandle = DelayHandle { inner: RefCell::new(None) };

// ———————————————————————————————————————————————————————————————————————————————————————
//                                         Init
// ———————————————————————————————————————————————————————————————————————————————————————
pub fn init(delay: CortexmDelay) {
  let mut inner = DELAY.inner.borrow_mut();
  if inner.is_some() {
    panic!("already initialized");
  }
  *inner = Some(delay);
}

pub struct DelayHandle {
  inner: RefCell<Option<CortexmDelay>>,
}

unsafe impl Sync for DelayHandle {}

impl DelayHandle {
  fn with_delay<F>(&self, f: F)
  where
    F: FnOnce(&mut CortexmDelay),
  {
    #[cfg(debug_assertions)]
    if cortex_m::peripheral::SCB::vect_active() != cortex_m::peripheral::scb::VectActive::ThreadMode
    {
      panic!("DELAY called from interrupt context!");
    }

    let mut cell = self.inner.borrow_mut();
    let delay = cell.as_mut().expect("DELAY not initialized");
    f(delay);
  }

  pub fn ms(&self, ms: u32) {
    self.with_delay(|inner| inner.delay_ms(ms));
  }

  pub fn us(&self, us: u32) {
    self.with_delay(|inner| inner.delay_us(us));
  }
}
