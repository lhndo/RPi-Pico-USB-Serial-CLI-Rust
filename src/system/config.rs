//! Configuration builder
//! Provides pin initialization, and data regarding aliases, gpio, and function groups

use core::fmt;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use rp2040_hal as hal;
//
use hal::gpio;
use hal::gpio::{AnyPin, DynPinId, DynPullType};
use hal::gpio::{FunctionNull, PullDown};

use heapless::Vec;
use once_cell::sync::Lazy;
use thiserror::Error;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::new(crate::pin_config::PIN_DEFINITION));

const PINOUT_CAPACITY: usize = 30;

pub type FullDynPinType = gpio::Pin<gpio::DynPinId, gpio::DynFunction, gpio::DynPullType>;
pub type RawDynPinType = gpio::Pin<DynPinId, FunctionNull, PullDown>;
pub type Result<T> = core::result::Result<T, Error>;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Config Structs
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Stores the device configuration.
pub struct Config {
  pub pins: Vec<PinDef, PINOUT_CAPACITY>,
}

impl Config {
  /// Creates a new Config instance containing the filtered list of pins.
  /// Does some basic safety checks to ensure a valid configuration.
  fn new(definition: &'static [Def]) -> Self {
    //

    // Pre-check pin id
    let mut seen = [false; PINOUT_CAPACITY];
    for def in definition {
      if let PinId::Gpio(id) = def.id {
        if id > 29 {
          panic!("pin out of bounds: {}", id);
        }

        if seen[id as usize] {
          panic!("duplicate config pin: {}", id); // duplicate found
        }
        seen[id as usize] = true;
      }
    }

    // Creating pin alias definitions
    let mut pins = Vec::<PinDef, PINOUT_CAPACITY>::new();

    for f_pin in definition.iter().filter(|def| def.id != PinId::NA) {
      let id = match f_pin.id {
        PinId::Gpio(id) => id,
        _ => unreachable!("config filter fail"),
      };

      pins
        .push(PinDef {
          alias: f_pin.alias,
          id,
          group: f_pin.group,
          taken: AtomicBool::new(false),
        })
        .ok()
        .expect("config build fail");
    }

    Self { pins }
  }

  /// Returns an iterator of GPIO num over all pins belonging to a specific group.
  pub fn get_group_iter(&self, group: Group) -> impl Iterator<Item = u8> {
    self.pins.iter().filter(move |pin| pin.group.eq(&group)).map(|pin| pin.id)
  }

  /// Gets the pin GPIO number associated with a given string alias.
  pub fn get_gpio(&self, alias: &str) -> Result<u8> {
    self
      .pins
      .iter()
      .find(|pin| pin.alias.eq_ignore_ascii_case(alias))
      .map(|pin| pin.id)
      .ok_or(Error::AliasNotFound)
  }

  /// Gets the string alias associated with a given pin GPIO number (`u8`).
  pub fn get_alias(&self, id: u8) -> Result<&'static str> {
    self
      .pins
      .iter()
      .find(|def| def.id == id)
      .map(|def| def.alias)
      .ok_or(Error::GpioNotFound)
  }

  /// Get the pin definition struct stored in the config
  pub fn get_pin_def_by_gpio(&self, id: u8) -> Result<&PinDef> {
    self.pins.iter().find(|pin| pin.id == id).ok_or(Error::GpioNotFound)
  }

  /// Get the pin definition struct stored in the config
  pub fn get_pin_def_by_alias(&self, alias: &str) -> Result<&PinDef> {
    self
      .pins
      .iter()
      .find(|pin| pin.alias.eq_ignore_ascii_case(alias))
      .ok_or(Error::AliasNotFound)
  }

  /// Gets the `Group` associated with a given pin GPIO number (`u8`).
  pub fn get_group_type(&self, id: u8) -> Option<Group> {
    self.pins.iter().find(|pin| pin.id == id).map(|def| def.group)
  }

  /// Getting gpio and alias as a pair based on the inputs provided.
  /// GPIO input has first choice if both are not None.
  pub fn get_gpio_alias_pair(&self, gpio: Option<u8>, alias: Option<&str>) -> Result<(u8, &str)> {
    if let Some(gpio_) = gpio {
      //  Getting alias from gpio
      let alias_ = self.get_alias(gpio_)?;
      Ok((gpio_, alias_))
    }
    // Getting gpio from alias
    else if let Some(alias_) = alias {
      let pin = self.get_pin_def_by_alias(alias_)?;
      Ok((pin.id, pin.alias))
    }
    else {
      // No Option was given
      Err(Error::GpioNotFound)
    }
  }

  /// Creates a DynPinId of the requested function and pull type, and marks the pin taken
  pub fn take_pin<F, P>(&self, id: u8) -> Option<gpio::Pin<DynPinId, F, P>>
  where
    F: gpio::Function,
    P: gpio::PullType,
  {
    let def = self.pins.iter().find(|pin| pin.id == id)?;

    if def.taken.load(Ordering::Relaxed) {
      return None; // already taken
    }

    let id = def.id;
    let pin: gpio::Pin<DynPinId, F, P> = new_pin_by_gpio_id(id)?;

    def.taken.store(true, Ordering::Relaxed);
    Some(pin)
  }

  /// Creates a DynPinId of the requested function and pull type, and marks the pin taken
  pub fn take_pin_by_alias<F, P>(&self, alias: &str) -> Result<gpio::Pin<DynPinId, F, P>>
  where
    F: gpio::Function,
    P: gpio::PullType,
  {
    let id = self.get_pin_def_by_alias(alias)?.id;
    self.take_pin(id).ok_or(Error::PinAlreadyConfigured)
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                       Internal Pin Definition
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Pin configuration layout.
pub struct PinDef {
  pub alias: &'static str,
  pub id:    u8,
  pub group: Group,
  pub taken: AtomicBool,
}

/// The functional group a pin belongs to
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Group {
  Reserved,
  Adc,
  Pwm,
  I2c,
  Spi,
  Uart,
  Inputs,
  Outputs,
  C1_Adc,
  C1_Pwm,
  C1_I2c,
  C1_Spi,
  C1_Uart,
  C1_Inputs,
  C1_Outputs,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                  Configuration Definition Structures
// —————————————————————————————————————————————————————————————————————————————————————————————————

// Pin configuration layout used only for the intial user provided definitions.
#[derive(Copy, Clone)]
pub struct Def {
  pub alias: &'static str,
  pub id:    PinId,
  pub group: Group,
}

// Pin gpio id definition
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PinId {
  Gpio(u8),
  NA,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Error
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum Error {
  #[error("gpio not found")]
  GpioNotFound,

  #[error("alias not found")]
  AliasNotFound,

  #[error("pin already configured")]
  PinAlreadyConfigured,

  #[error("pin out of bounds")]
  OutOfBounds,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

impl fmt::Display for Group {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:#?}", self)
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Free Functions
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Converts concrete pin into a fully dynamic pin
pub fn pin_into_full_dynamic<P: AnyPin>(pin: P) -> FullDynPinType {
  let pin: gpio::SpecificPin<P> = pin.into();
  pin.into_dyn_pin().into_function().into_pull_type::<DynPullType>()
}

/// Creates a dynamic pin with concrete functions based on gpio id
/// User must make sure no other that pin exists at the same time.
fn new_pin_by_gpio_id<F, P>(gpio_id: u8) -> Option<gpio::Pin<DynPinId, F, P>>
where
  F: gpio::Function,
  P: gpio::PullType,
{
  if gpio_id > 29 {
    panic!("GPIO > 29")
  }

  // TODO: check for function

  let pin = unsafe {
    gpio::new_pin(gpio::DynPinId {
      bank: gpio::DynBankId::Bank0,
      num:  gpio_id,
    })
  };

  pin.try_into_function::<F>().ok().map(|p| p.into_pull_type::<P>())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Macros
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[macro_export]
macro_rules! gpio {
  ($alias:ident) => {
    $crate::system::config::CONFIG.get_gpio(stringify!($alias)).unwrap()
  };
}
