//! Analog-Digital Converter (ADC) Wrapper for the RP2040 microcontroller

use embedded_hal_0_2::adc::OneShot;
use rp2040_hal as hal;

//
use hal::adc::{Adc, AdcPin, TempSense};
use hal::gpio;

pub const ADC_BITS: u32 = 12;
pub const ADC_MAX: f32 = ((1 << ADC_BITS) - 1) as f32;
pub const ADC_VREF: f32 = 3.3;

pub const TEMP_SENSE_CHN: u8 = 4;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Adcs
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub type DynPinType = gpio::Pin<gpio::DynPinId, gpio::DynFunction, gpio::DynPullType>;

pub struct Adcs {
  pub hal_adc:    Adc,
  pub temp_sense: TempSense,
  pub adc0:       Option<AdcPin<DynPinType>>,
  pub adc1:       Option<AdcPin<DynPinType>>,
  pub adc2:       Option<AdcPin<DynPinType>>,
  pub adc3:       Option<AdcPin<DynPinType>>,
}

impl Adcs {
  pub fn new(hal_adc: Adc, temp_sense: TempSense) -> Self {
    Self {
      hal_adc,
      temp_sense,
      adc0: None,
      adc1: None,
      adc2: None,
      adc3: None,
    }
  }

  pub fn register(&mut self, pin: DynPinType) {
    let pin_id = pin.id().num;

    match pin_id {
      26 => self.adc0 = AdcPin::new(pin).ok(),
      27 => self.adc1 = AdcPin::new(pin).ok(),
      28 => self.adc2 = AdcPin::new(pin).ok(),
      29 => self.adc3 = AdcPin::new(pin).ok(),
      _ => unreachable!("pin id not adc valid"),
    }

    // Store the AdcPin in the struct
  }

  /// Returns the main HAL ADC object
  pub fn get_adc(&mut self) -> &mut Adc {
    &mut self.hal_adc
  }

  /// One shot read of the ADC channel 0-3, and 4 as TEMP_SENSE channel
  /// Returns Some or None
  pub fn read(&mut self, id: u8) -> Option<u16> {
    match id {
      0 => self.adc0.as_mut().and_then(|pin| self.hal_adc.read(pin).ok()),
      1 => self.adc1.as_mut().and_then(|pin| self.hal_adc.read(pin).ok()),
      2 => self.adc2.as_mut().and_then(|pin| self.hal_adc.read(pin).ok()),
      3 => self.adc3.as_mut().and_then(|pin| self.hal_adc.read(pin).ok()),
      TEMP_SENSE_CHN => self.hal_adc.read(&mut self.temp_sense).ok(),
      _ => None,
    }
  }

  /// One shot read based on the Pin ID (4 as TEMP_SENSE ID)
  pub fn read_by_gpio_id(&mut self, gpio: u8) -> Option<u16> {
    match gpio {
      26 => self.read(0),
      27 => self.read(1),
      28 => self.read(2),
      29 => self.read(3),
      TEMP_SENSE_CHN => self.read(TEMP_SENSE_CHN),
      _ => None,
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ——————————————————————————————————————— Adc Conversions —————————————————————————————————————————
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
    // "ref_res / x" // If you ref resistor to Gnd instead of V+
    ref_res as f32 * x
  }
}
