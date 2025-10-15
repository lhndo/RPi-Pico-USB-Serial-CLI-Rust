//! Pulse Width Modulation (PWM) Wrapper for the RP2040 microcontroller

use core::convert::Infallible;
use core::fmt;

use super::config::Error;
use super::config::Result;

use embedded_hal::pwm::SetDutyCycle;

use rp2040_hal as hal;
//
use hal::gpio;
use hal::pwm;

use heapless::Vec;

const MAX_PWM_PINS: usize = 16;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Pwms
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub type PwmPinType = gpio::Pin<gpio::DynPinId, gpio::FunctionPwm, gpio::PullNone>;

pub struct Pwms {
  pub pwm0:    PwmSlice<pwm::Pwm0>,
  pub pwm1:    PwmSlice<pwm::Pwm1>,
  pub pwm2:    PwmSlice<pwm::Pwm2>,
  pub pwm3:    PwmSlice<pwm::Pwm3>,
  pub pwm4:    PwmSlice<pwm::Pwm4>,
  pub pwm5:    PwmSlice<pwm::Pwm5>,
  pub pwm6:    PwmSlice<pwm::Pwm6>,
  pub pwm7:    PwmSlice<pwm::Pwm7>,
  pwm_aliases: Vec<PwmAlias, MAX_PWM_PINS>,
}

struct PwmAlias {
  gpio_id:  u8,
  slice_id: u8,
  channel:  Channel,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Channel {
  A,
  B,
}

impl fmt::Display for Channel {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
    write!(f, "{:#?}", self)
  }
}

// ————————————————————————————————————————— Pwms Impl ————————————————————————————————————————————

impl Pwms {
  pub fn new(slices: pwm::Slices, sys_clk_hz: u32, default_freq: u32) -> Self {
    Pwms {
      pwm0:        PwmSlice::new(slices.pwm0, default_freq, false, sys_clk_hz),
      pwm1:        PwmSlice::new(slices.pwm1, default_freq, false, sys_clk_hz),
      pwm2:        PwmSlice::new(slices.pwm2, default_freq, false, sys_clk_hz),
      pwm3:        PwmSlice::new(slices.pwm3, default_freq, false, sys_clk_hz),
      pwm4:        PwmSlice::new(slices.pwm4, default_freq, false, sys_clk_hz),
      pwm5:        PwmSlice::new(slices.pwm5, default_freq, false, sys_clk_hz),
      pwm6:        PwmSlice::new(slices.pwm6, default_freq, false, sys_clk_hz),
      pwm7:        PwmSlice::new(slices.pwm7, default_freq, false, sys_clk_hz),
      pwm_aliases: Vec::new(),
    }
  }

  // Direct PWM pin output registration bypassing type safe HAL limitation
  pub fn register(&mut self, pin: PwmPinType) {
    // Getting gpio id
    let gpio_id = pin.id().num;

    // Calculate slice and channel from GPIO number
    let slice_id = (gpio_id >> 1) & 0x7;
    let channel = if gpio_id & 1 == 0 { Channel::A } else { Channel::B };

    // Creating pins directly due to HAL type restrictions
    unsafe {
      let io_bank0 = &(*hal::pac::IO_BANK0::ptr());
      let current_func = io_bank0.gpio(gpio_id as usize).gpio_ctrl().read().funcsel().bits();

      // Function codes on RP2040:
      // 0 = XIP (flash)
      // 1 = SPI
      // 2 = UART
      // 3 = I2C
      // 4 = PWM
      // 5 = SIO (regular GPIO)
      // 6-8 = PIO0-2
      // 9 = CLOCK
      // 31 = NULL (reset/unused)

      match current_func {
        4 => {}
        _ => {
          unreachable!("PWM pin in use");
        }
      }

      io_bank0.gpio(gpio_id as usize).gpio_ctrl().write(|w| w.funcsel().pwm());
    }

    // Creating Pin Alias used for slice and channel retrival by gpio id
    let _ = self.pwm_aliases.push(PwmAlias {
      gpio_id,
      slice_id,
      channel,
    });
  }

  /// Returns Slice ID AND Channel associated with the gpio pin
  pub fn get_pwm_slice_id_by_gpio(&self, gpio: u8) -> Result<(u8, Channel)> {
    let alias = self
      .pwm_aliases
      .iter()
      .find(|alias| alias.gpio_id == gpio)
      .ok_or(Error::GpioNotFound)?;
    Ok((alias.slice_id, alias.channel))
  }

  /// Get PWM Slice Channel from GPIO id
  pub fn get_channel_by_gpio(
    &mut self,
    gpio: u8,
  ) -> Result<&mut dyn SetDutyCycle<Error = Infallible>> {
    //
    let (slice_id, channel) = self.get_pwm_slice_id_by_gpio(gpio)?;

    Ok(match (slice_id, channel) {
      (0, Channel::A) => self.pwm0.get_channel_a(),
      (0, Channel::B) => self.pwm0.get_channel_b(),
      (1, Channel::A) => self.pwm1.get_channel_a(),
      (1, Channel::B) => self.pwm1.get_channel_b(),
      (2, Channel::A) => self.pwm2.get_channel_a(),
      (2, Channel::B) => self.pwm2.get_channel_b(),
      (3, Channel::A) => self.pwm3.get_channel_a(),
      (3, Channel::B) => self.pwm3.get_channel_b(),
      (4, Channel::A) => self.pwm4.get_channel_a(),
      (4, Channel::B) => self.pwm4.get_channel_b(),
      (5, Channel::A) => self.pwm5.get_channel_a(),
      (5, Channel::B) => self.pwm5.get_channel_b(),
      (6, Channel::A) => self.pwm6.get_channel_a(),
      (6, Channel::B) => self.pwm6.get_channel_b(),
      (7, Channel::A) => self.pwm7.get_channel_a(),
      (7, Channel::B) => self.pwm7.get_channel_b(),
      _ => return Err(Error::GpioNotFound), // Invalid slice_id
    })
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            PwmSlice
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// PwmSlice Wrapper
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
  pub sys_clk_hz: u32,
}

// ———————————————————————————————————————— PwmSlice impl ——————————————————————————————————————————

impl<I> PwmSlice<I>
where
  I: pwm::SliceId,
  <I as pwm::SliceId>::Reset: pwm::ValidSliceMode<I>,
{
  fn new(
    slice: pwm::Slice<I, <I as pwm::SliceId>::Reset>,
    freq: u32,
    ph_correct: bool,
    sys_clk_hz: u32,
  ) -> Self {
    let mut slice = PwmSlice {
      slice,
      freq,
      ph_correct,
      enabled: false,
      sys_clk_hz,
    };

    slice.set_freq(freq);
    slice
  }

  /// Sets pwm slice frequency and resets duty cycle to 50%
  pub fn set_freq(&mut self, freq: u32) {
    self.slice.disable();

    if freq == 0 {
      self.enabled = false;
      return;
    }

    self.freq = freq;
    let top = self.slice.get_top();
    let (int, frac) = calculate_pwm_dividers(self.sys_clk_hz, freq, top, self.ph_correct);
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
    }
    else {
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

  /// Only use for functions not covered by this wrapper.
  /// Don't set enable, freq, ph_correct, top directly
  pub fn get_pwm_slice(&mut self) -> &mut pwm::Slice<I, <I as pwm::SliceId>::Reset> {
    &mut self.slice
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

  /// Get PWM Slice Channel as SetDutyCycle trait
  pub fn get_channel(&mut self, channel: Channel) -> &mut dyn SetDutyCycle<Error = Infallible> {
    match channel {
      Channel::A => self.get_channel_a(),
      Channel::B => self.get_channel_b(),
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ————————————————————————————————————————— Pwm Channel ———————————————————————————————————————————

pub trait PwmChannelExt {
  fn set_duty_cycle_us(&mut self, us: u16, freq_hz: u32);
}

impl<C: SetDutyCycle> PwmChannelExt for C {
  fn set_duty_cycle_us(&mut self, duty_us: u16, freq_hz: u32) {
    let duty = calculate_duty_from_us(duty_us, freq_hz, self.max_duty_cycle());
    let _ = self.set_duty_cycle(duty);
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Free Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Calculate duty cycle from us and frequency
pub fn calculate_duty_from_us(duty_us: u16, freq_hz: u32, max_duty: u16) -> u16 {
  const MAX_U16: u32 = u16::MAX as u32;
  let freq_us = if freq_hz > 0 { 1_000_000 / freq_hz } else { 0 };

  if freq_us == 0 || duty_us == 0 {
    return 0;
  }

  if duty_us as u32 >= freq_us {
    return max_duty;
  }

  let (num, den) = if freq_us > MAX_U16 {
    // Period is too large for a u16; scale both values down.
    let scaler = freq_us / MAX_U16 + 1;
    ((duty_us as u32 / scaler) as u16, (freq_us / scaler) as u16)
  }
  else {
    // Period fits, no scaling needed.
    (duty_us, freq_us as u16)
  };

  debug_assert!(num <= den);
  debug_assert!(den != 0);

  let duty = u32::from(num) * u32::from(max_duty) / u32::from(den);

  duty as u16
}

/// Calculates pwm int and frac clock dividers based on sys clock, top, and desired hz frequency
pub fn calculate_pwm_dividers(sys_clk_hz: u32, hz: u32, top: u16, phase_correct: bool) -> (u8, u8) {
  let hz = if phase_correct { hz * 2 } else { hz };
  let denominator = hz * (top as u32 + 1);
  // Add half denominator before division for proper rounding
  let divider_x16 = (sys_clk_hz * 16 + denominator / 2) / denominator;

  // Clamp to valid range: 1.0 to 255.9375 (16 to 4095 in fixed-point)
  let clamped = divider_x16.clamp(16, 4095);

  // Compile-time optimized to bit shifts
  let div_int = (clamped >> 4) as u8; // Divide by 16
  let div_frac = (clamped & 0xF) as u8; // Modulo 16

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
