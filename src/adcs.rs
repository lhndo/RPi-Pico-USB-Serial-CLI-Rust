use crate::device::{ADC_MAX, ADC_VREF};

use cortex_m::prelude::_embedded_hal_adc_OneShot;
use rp_pico::hal::adc::{Adc, AdcPin, TempSense};
use rp_pico::hal::gpio;

pub const TEMP_SENSE_CHN: u8 = 255;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Adcs
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub type AcdPinType<T> = AdcPin<gpio::Pin<T, gpio::FunctionSio<gpio::SioInput>, gpio::PullNone>>;

pub struct Acds {
  pub hal_adc:    Adc,
  pub temp_sense: TempSense,
  pub adc0:       AcdPinType<gpio::bank0::Gpio26>,
  pub adc1:       AcdPinType<gpio::bank0::Gpio27>,
  pub adc2:       AcdPinType<gpio::bank0::Gpio28>,
  pub adc3:       AcdPinType<gpio::bank0::Gpio29>,
}

impl Acds {
  /// One shot read of the ADC channel 0-3, and 255 (as TEMP_SENSE_CHN)
  /// Returns Some or None
  pub fn read_channel(&mut self, id: u8) -> Option<u16> {
    match id {
      0 => self.hal_adc.read(&mut self.adc0).unwrap_or(None),
      1 => self.hal_adc.read(&mut self.adc1).unwrap_or(None),
      2 => self.hal_adc.read(&mut self.adc2).unwrap_or(None),
      3 => self.hal_adc.read(&mut self.adc3).unwrap_or(None),
      255 => self.hal_adc.read(&mut self.temp_sense).unwrap_or(None),
      _ => None,
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ————————————————————————————————————————— Adc Tools ———————————————————————————————————————————

pub trait AdcConversion {
  /// Convert raw u16 ADC reading to volts.
  fn to_voltage(&self) -> f32;
  fn to_resistance(&self, ref_res: u32) -> f32;
}

// Impl for u16, assuming 12-bit ADC (0..=4095) and 3.3 V reference.
impl AdcConversion for u16 {
  fn to_voltage(&self) -> f32 {
    (*self as f32) * ADC_VREF / ADC_MAX
  }

  fn to_resistance(&self, ref_res: u32) -> f32 {
    let x: f32 = (ADC_MAX / *self as f32) - 1.0;
    // ref_res / x // If you ref resistor to Gnd instead of V+
    ref_res as f32 * x
  }
}
