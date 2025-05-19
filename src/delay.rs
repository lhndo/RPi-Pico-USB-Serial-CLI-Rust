// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          Delay Handle
// ————————————————————————————————————————————————————————————————————————————————————————————————
#![allow(static_mut_refs)]

use cortex_m::delay::Delay;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————


pub static DELAY: DelayHandle = DelayHandle;
static mut GLOBAL_DELAY: Option<Delay> = None;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     Delay Handle Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct DelayHandle;

impl DelayHandle {
  pub fn init(&self, delay: Delay) {
    unsafe {
      GLOBAL_DELAY.replace(delay);
    }
  }

  /// Unsafe since it is sigular
  pub fn delay_ms(&self, ms: u32) {
    unsafe {
      if let Some(delay) = GLOBAL_DELAY.as_mut() {
        delay.delay_ms(ms);
      }
    }
  }


  pub fn delay_us(&self, us: u32) {
    unsafe {
      if let Some(delay) = GLOBAL_DELAY.as_mut() {
        delay.delay_us(us);
      }
    }
  }
}
