use core::sync::atomic::Ordering;

use crate::device::SYS_CLK_HZ;

use embedded_hal::pwm::SetDutyCycle;
use rp_pico::hal::pwm;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Pwms
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Pwms {
  pub pwm0: PwmSlice<pwm::Pwm0>,
  pub pwm1: PwmSlice<pwm::Pwm1>,
  pub pwm2: PwmSlice<pwm::Pwm2>,
  pub pwm3: PwmSlice<pwm::Pwm3>,
  pub pwm4: PwmSlice<pwm::Pwm4>,
  pub pwm5: PwmSlice<pwm::Pwm5>,
  pub pwm6: PwmSlice<pwm::Pwm6>,
  pub pwm7: PwmSlice<pwm::Pwm7>,
}

// ————————————————————————————————————————— Pwms Impl ————————————————————————————————————————————

impl Pwms {
  pub fn new(slices: pwm::Slices, default_freq: u32) -> Self {
    Pwms {
      pwm0: PwmSlice::new(slices.pwm0, default_freq, false),
      pwm1: PwmSlice::new(slices.pwm1, default_freq, false),
      pwm2: PwmSlice::new(slices.pwm2, default_freq, false),
      pwm3: PwmSlice::new(slices.pwm3, default_freq, false),
      pwm4: PwmSlice::new(slices.pwm4, default_freq, false),
      pwm5: PwmSlice::new(slices.pwm5, default_freq, false),
      pwm6: PwmSlice::new(slices.pwm6, default_freq, false),
      pwm7: PwmSlice::new(slices.pwm7, default_freq, false),
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            PwmSlice
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// PwmSlice Handler
/// Set freq, ph_correct, and enable status though its methods, and not directly on the slice member
pub struct PwmSlice<I>
where
  I: pwm::SliceId,
  <I as pwm::SliceId>::Reset: pwm::ValidSliceMode<I>,
{
  /// The specific PWM slice (e.g., Pwm0, Pwm1) in its initial Reset state.
  pub slice:      pwm::Slice<I, <I as pwm::SliceId>::Reset>,
  pub freq:       u32,
  pub ph_correct: bool,
  pub enabled:    bool,
}

// ———————————————————————————————————————— PwmSlice impl ——————————————————————————————————————————

impl<I> PwmSlice<I>
where
  I: pwm::SliceId,
  <I as pwm::SliceId>::Reset: pwm::ValidSliceMode<I>,
{
  pub fn new(
    slice: pwm::Slice<I, <I as pwm::SliceId>::Reset>,
    freq: u32,
    ph_correct: bool,
  ) -> Self {
    let mut slice = PwmSlice {
      slice,
      freq,
      ph_correct,
      enabled: false,
    };

    slice.set_freq(freq);

    slice
  }

  /// Sets pwm slice frequency and resets duty cycle to 50%
  pub fn set_freq(&mut self, freq: u32) {
    self.slice.disable();

    self.freq = freq;
    let top = self.slice.get_top();
    let (int, frac) = calculate_pwm_dividers(freq, top, self.ph_correct);
    self.slice.set_div_int(int);
    self.slice.set_div_frac(frac);

    let _ = self.get_channel_a().set_duty_cycle_percent(50);
    let _ = self.get_channel_b().set_duty_cycle_percent(50);

    if self.enabled {
      self.slice.enable();
    }
  }

  pub fn set_ph_correct(&mut self, enable: bool) {
    if enable == self.ph_correct {
      return;
    }
    self.ph_correct = enable;

    if enable {
      self.slice.set_ph_correct();
    } else {
      self.slice.clr_ph_correct();
    }
    self.set_freq(self.freq);
  }

  pub fn set_top(&mut self, top: u16) {
    self.slice.set_top(top);
    self.set_freq(self.freq);
  }

  pub fn enable(&mut self) {
    self.enabled = true;
    self.slice.enable();
  }

  pub fn disable(&mut self) {
    self.enabled = false;
    self.slice.disable();
  }

  pub fn get_channel_a(
    &mut self,
  ) -> &mut pwm::Channel<pwm::Slice<I, <I as pwm::SliceId>::Reset>, pwm::A> {
    &mut self.slice.channel_a
  }

  pub fn get_channel_b(
    &mut self,
  ) -> &mut pwm::Channel<pwm::Slice<I, <I as pwm::SliceId>::Reset>, pwm::B> {
    &mut self.slice.channel_b
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub trait PwmChannelExt {
  fn set_duty_cycle_us(&mut self, us: u16, freq_hz: u32);
}

impl<T: SetDutyCycle> PwmChannelExt for T {
  fn set_duty_cycle_us(&mut self, duty_us: u16, freq_hz: u32) {
    let freq_us = if freq_hz > 0 { 1_000_000 / freq_hz } else { 0 };

    if freq_us == 0 && duty_us == 0 {
      let _ = self.set_duty_cycle(0);
      return;
    }

    if duty_us as u32 >= freq_us {
      let _ = self.set_duty_cycle(self.max_duty_cycle());
      return;
    }

    const MAX_U16: u32 = u16::MAX as u32;

    let (num, den) = if freq_us > MAX_U16 {
      // Period is too large for a u16; scale both values down.
      let scaler = freq_us / MAX_U16 + 1;
      ((duty_us as u32 / scaler) as u16, (freq_us / scaler) as u16)
    } else {
      // Period fits, no scaling needed.
      (duty_us, freq_us as u16)
    };

    let _ = self.set_duty_cycle_fraction(num, den);
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Free Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Calculates pwm int and frac clock dividers based on sys clock, top, and desired hz frequency
pub fn calculate_pwm_dividers(hz: u32, top: u16, phase_correct: bool) -> (u8, u8) {
  let hz = if phase_correct { hz * 2 } else { hz };
  let divider = SYS_CLK_HZ.load(Ordering::Relaxed) as f32 / (hz as f32 * (top as f32 + 1.0));
  let clamped_divider = divider.clamp(1.0, 255.9375);

  let div_int = (clamped_divider + 0.5) as u8;
  let div_frac = ((clamped_divider - div_int as f32) * 16.0 + 0.5) as u8;

  (div_int, div_frac)
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Macro
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[macro_export]
macro_rules! with_pwm_slice {
  ($self:expr, $id:expr, | $slice:ident | $body:expr) => {
    match $id {
      0 => {
        let $slice = &mut $self.pwm0;
        $body
      }
      1 => {
        let $slice = &mut $self.pwm1;
        $body
      }
      2 => {
        let $slice = &mut $self.pwm2;
        $body
      }
      3 => {
        let $slice = &mut $self.pwm3;
        $body
      }
      4 => {
        let $slice = &mut $self.pwm4;
        $body
      }
      5 => {
        let $slice = &mut $self.pwm5;
        $body
      }
      6 => {
        let $slice = &mut $self.pwm6;
        $body
      }
      7 => {
        let $slice = &mut $self.pwm7;
        $body
      }
      // ... other match arms
      _ => panic!("Invalid PWM Slice ID"),
    }
  };
}
