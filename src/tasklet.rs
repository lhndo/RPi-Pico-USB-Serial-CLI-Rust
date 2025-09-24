//! A tasklet utility that allows creation and execution of timed tasks
//!
//! To be used in main program loops.
//!
//!
//! Example - Non blocking timer based task:
//! ```no_run
//!   
//! let mut ledtask = Tasklet::new(interval as u32, times * 2, &device.timer);
//!
//! while !ledtask.is_exhausted() {
//!   if ledtask.is_ready() {
//!     led.toggle().unwrap();
//!
//!     if led.is_set_high().unwrap() {
//!       print!("Blink {} | ", blink);
//!       blink += 1;
//!     }
//!   }
//! ```

use embedded_hal_0_2::timer::{Cancel, CountDown as CountDownT};
use hal::fugit::{ExtU32, MicrosDurationU32};
use hal::timer::{CountDown, Timer};
use rp2040_hal as hal;

/// Non blocking periodic task for in-loop usage
pub struct Tasklet {
  count_down:     CountDown,
  interval:       MicrosDurationU32,
  initial_runs:   u16,
  remaining_runs: u16,
  is_first_poll:  bool,
}

impl<'a> Tasklet {
  /// Create a new task. Runs: 0 equals infinite
  #[inline]
  pub fn new(interval_ms: u32, runs: u16, timer: &'a Timer) -> Self {
    Tasklet {
      count_down:     timer.count_down(),
      interval:       (interval_ms * 1000).micros(),
      initial_runs:   runs,
      remaining_runs: runs,
      is_first_poll:  true,
    }
  }

  /// Polls the task. Returns `true` if the period has elapsed OR on the very first call.
  #[inline]
  pub fn is_ready(&mut self) -> bool {
    if self.is_first_poll {
      self.is_first_poll = false;
      self.count_down.start(self.interval);
      if self.initial_runs != 0 {
        self.remaining_runs -= 1;
      }
      return true;
    }

    if self.initial_runs != 0 && self.remaining_runs == 0 {
      return false;
    }

    if self.count_down.wait().is_ok() {
      if self.initial_runs != 0 {
        self.remaining_runs -= 1;
        if self.remaining_runs == 0 {
          let _ = self.count_down.cancel();
        }
      }
      true
    }
    else {
      false
    }
  }

  /// Resets the task
  #[inline]
  pub fn reset(&mut self) {
    self.remaining_runs = self.initial_runs;
    let _ = self.count_down.cancel();
    self.is_first_poll = true;
  }

  /// Cancels the task and stops it from firing
  #[inline]
  pub fn cancel(&mut self) -> Result<(), &'static str> {
    self.count_down.cancel()
  }

  /// Check to see if the no of runs have finished
  #[inline]
  pub fn is_exhausted(&self) -> bool {
    self.initial_runs != 0 && self.remaining_runs == 0
  }
}
