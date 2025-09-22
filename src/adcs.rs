use embedded_hal_0_2::adc::OneShot;
use rp_pico::hal::adc::{self, Adc, AdcPin, TempSense};
use rp_pico::hal::gpio;

use duplicate::duplicate_item;
use pastey::paste;

pub const ADC_BITS: u32 = 12;
pub const ADC_MAX: f32 = ((1 << ADC_BITS) - 1) as f32;
pub const ADC_VREF: f32 = 3.3;

pub const TEMP_SENSE_CHN: u8 = 4;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Adcs
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub type AdcPinType<T> = AdcPin<gpio::Pin<T, gpio::FunctionNull, gpio::PullDown>>;
pub type RawPin<T> = gpio::Pin<T, gpio::FunctionNull, gpio::PullDown>;

pub struct Adcs {
  hal_adc:    Adc,
  temp_sense: TempSense,
  adc0:       Option<AdcPinType<gpio::bank0::Gpio26>>,
  adc1:       Option<AdcPinType<gpio::bank0::Gpio27>>,
  adc2:       Option<AdcPinType<gpio::bank0::Gpio28>>,
  adc3:       Option<AdcPinType<gpio::bank0::Gpio29>>,
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

  // Generating pub fn set_adc0()... Used creating Analog Pins. 
  #[duplicate_item(
    adc_num   gpio_num; 
    [0]       [26];
    [1]       [27];
    [2]       [28];
    [3]       [29];

    )]
  paste! {
      pub fn [<set_adc adc_num>](&mut self, pin: RawPin<gpio::bank0::[<Gpio gpio_num>]>) -> &mut Self {
          self.[<adc adc_num>] = AdcPin::new(pin).ok();
          self
      }
  }


  /// One shot read of the ADC channel 0-3, and 4 as TEMP_SENSE channel
  /// Returns Some or None
  pub fn read_channel(&mut self, id: u8) -> Option<u16> {
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
      26 => self.read_channel(0),
      27 => self.read_channel(1),
      28 => self.read_channel(2),
      29 => self.read_channel(3),
      TEMP_SENSE_CHN => self.read_channel(TEMP_SENSE_CHN),
      _ => None,
    }
  }

  /// Returns the main HAL ADC object
  pub fn get_hal_adc(&mut self) -> &mut Adc {
    &mut self.hal_adc
  }

  /// Returns ADC Channel by ADC channel id as dyn AdcChannel  
  pub fn as_dyn_adc_channel(&mut self, id: u8) -> Option<&mut dyn adc::AdcChannel> {
    #[allow(clippy::option_map_or_none)] // Needed for Option to dyn recast to work
    match id {
      0 => self.adc0.as_mut().map_or(None, |a| Some(a)),
      1 => self.adc1.as_mut().map_or(None, |a| Some(a)),
      2 => self.adc2.as_mut().map_or(None, |a| Some(a)),
      3 => self.adc3.as_mut().map_or(None, |a| Some(a)),
      TEMP_SENSE_CHN => Some(&mut self.temp_sense),
      _ => None,
    }
  }

  pub fn get_adc0(&mut self) -> Option<&mut AdcPinType<gpio::bank0::Gpio26>> {
    self.adc0.as_mut()
  }

  pub fn get_adc1(&mut self) -> Option<&mut AdcPinType<gpio::bank0::Gpio27>> {
    self.adc1.as_mut()
  }

  pub fn get_adc2(&mut self) -> Option<&mut AdcPinType<gpio::bank0::Gpio28>> {
    self.adc2.as_mut()
  }

  pub fn get_adc3(&mut self) -> Option<&mut AdcPinType<gpio::bank0::Gpio29>> {
    self.adc3.as_mut()
  }

  pub fn get_temp_sense(&mut self) -> &mut TempSense {
    &mut self.temp_sense
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
