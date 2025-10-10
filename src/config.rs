//! Configuration
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
//                                        Pin Configuration
// —————————————————————————————————————————————————————————————————————————————————————————————————

// RPi Pico           - https://pico.pinout.xyz
// WeAct Studio RP2040 - https://mischianti.org/wp-content/uploads/2022/09/weact-studio-rp2040-raspberry-pi-pico-alternative-pinout-high-resolution.png
//
//                                                     --RPi Pico--
//                                                      ___USB___
// (PWM0 A)(UART0  TX)(I2C0 SDA)(SPI0  RX)   GP0  |  1 |o       o| 40 | VBUS 5V
// (PWM0 B)(UART0  RX)(I2C0 SCL)(SPI0 CSn)   GP1  |  2 |o       o| 39 | VSYS 5V*
//                                           GND  |  3 |o       o| 38 | GND
// (PWM1 A)(UART0 CTS)(I2C1 SDA)(SPI0 SCK)   GP2  |  4 |o       o| 37 | 3V3  En
// (PWM1 B)(UART0 RTS)(I2C1 SCL)(SPI0  TX)   GP3  |  5 |o       o| 36 | 3V3  Out
// (PWM2 A)(UART1  TX)(I2C0 SDA)(SPI0  RX)   GP4  |  6 |o       o| 35 | ADC  VREF
// (PWM2 B)(UART1  RX)(I2C0 SCL)(SPI0 CSn)   GP5  |  7 |o       o| 34 | GP28 A2    (SPI1  RX)(I2C0 SDA)(UART0  TX)(PWM6 A)
//                                           GND  |  8 |o       o| 33 | ADC  GND
// (PWM3 A)(UART1 CTS)(I2C1 SDA)(SPI1 SCK)   GP6  |  9 |o       o| 32 | GP27 A1    (SPI1  TX)(I2C1 SCL)(UART1 RTS)(PWM5 B)
// (PWM3 B)(UART1 RTS)(I2C1 SCL)(SPI1  TX)   GP7  | 10 |o       o| 31 | GP26 A0    (SPI1 SCK)(I2C1 SDA)(UART1 CTS)(PWM5 A)
// (PWM4 A)(UART1  TX)(I2C0 SDA)(SPI1  RX)   GP8  | 11 |o       o| 30 | RUN
// (PWM4 B)(UART1  RX)(I2C0 SCL)(SPI1 CSn)   GP9  | 12 |o       o| 29 | GP22       (SPI0 SCK)(I2C1 SDA)(UART1 CTS)(PWM3 A)
//                                           GND  | 13 |o       o| 28 | GND
// (PWM5 A)(UART1 CTS)(I2C1 SDA)(SPI1 SCK)   GP10 | 14 |o       o| 27 | GP21       (SPI0 CSn)(I2C0 SCL)(UART1  RX)(PWM2 B)
// (PWM5 B)(UART1 RTS)(I2C1 SCL)(SPI1  TX)   GP11 | 15 |o       o| 26 | GP20       (SPI0  RX)(I2C0 SDA)(UART1  TX)(PWM2 A)
// (PWM6 A)(UART0  TX)(I2C0 SDA)(SPI1  RX)   GP12 | 16 |o       o| 25 | GP19       (SPI0  TX)(I2C1 SCL)(UART0 RTS)(PWM1 B)
// (PWM6 B)(UART0  RX)(I2C0 SCL)(SPI1 CSn)   GP13 | 17 |o       o| 24 | GP18       (SPI0 SCK)(I2C1 SDA)(UART0 CTS)(PWM1 A)
//                                           GND  | 18 |o       o| 23 | GND
// (PWM7 A)(UART0 CTS)(I2C1 SDA)(SPI1 SCK)   GP14 | 19 |o       o| 22 | GP17       (SPI0 CSn)(I2C0 SCL)(UART0  RX)(PWM0 B)
// (PWM7 B)(UART0 RTS)(I2C1 SCL)(SPI1  TX)   GP15 | 20 |o__ooo__o| 21 | GP16       (SPI0  RX)(I2C0 SDA)(UART0  TX)(PWM0 A)
//
//                                             --[ SWD: CLK, GND, DIO ]--
//
// | Pin     | Description           | Notes                                                                                  |
// |---------|-----------------------|----------------------------------------------------------------------------------------|
// | VSYS*   | System voltage in/out | 5V out when powered by USB (diode to VBUS), 1.8V to 5.5V in if powered externally      |
// | 3V3 Out | Chip 3V3 supply       | Can be used to power external circuitry, recommended to keep the load less than 300mA  |
// | GP23    | RT6150B-33GQW P-Select| LOW (def) high efficiency (PFM), HIGH improved ripple (PWM)  | WeAct - extra Button    |
// | GP24    | VBUS Sense            | Detect USB power or VBUS pin                                 | Weact - extra GPIO      |
// | GP25    | User LED              |                                                                                        |
// | GP29 A3 | VSYS Sense            | Read VSYS/3 through resistor divider and FET Q1              | WeAct - extra GPIO A3   |
// | A4      | Temperature           | Read onboard temperature sensor                                                        |

#[rustfmt::skip]
const PIN_DEFINITION: &[Def] = {
  use Group::*;
  use PinId::*;

    &[
        //           Alias       GPIO            Group           Valid Pins
        // ADC
        Def { alias: "ADC0",     id: Gpio(26), group: Adc    }, // GP26
        Def { alias: "ADC1",     id: Gpio(27), group: Adc    }, // GP27
        Def { alias: "ADC2",     id: Gpio(28), group: Adc    }, // GP28
        Def { alias: "ADC3",     id: Gpio(29), group: Adc    }, // GP29

        // PWM
        Def { alias: "PWM0_A",   id: NA,       group: Pwm    }, // GP0, GP16
        Def { alias: "PWM0_B",   id: NA,       group: Pwm    }, // GP1, GP17
        Def { alias: "PWM1_A",   id: NA,       group: Pwm    }, // GP2, GP18
        Def { alias: "PWM1_B",   id: NA,       group: Pwm    }, // GP3, GP19
        Def { alias: "PWM2_A",   id: NA,       group: Pwm    }, // GP4, GP20
        Def { alias: "PWM2_B",   id: Gpio(21), group: Pwm    }, // GP5, GP21s
        Def { alias: "PWM3_A",   id: Gpio(6),  group: Pwm    }, // GP6, GP22
        Def { alias: "PWM3_B",   id: NA,       group: Pwm    }, // GP7
        Def { alias: "PWM4_A",   id: Gpio(8),  group: Pwm    }, // GP8
        Def { alias: "PWM4_B",   id: NA,       group: Pwm    }, // GP9
        Def { alias: "PWM5_A",   id: NA,       group: Pwm    }, // GP10, GP26
        Def { alias: "PWM5_B",   id: NA,       group: Pwm    }, // GP11, GP27
        Def { alias: "PWM6_A",   id: NA,       group: Pwm    }, // GP12, GP28
        Def { alias: "PWM6_B",   id: NA,       group: Pwm    }, // GP13
        Def { alias: "PWM7_A",   id: NA,       group: Pwm    }, // GP14
        Def { alias: "PWM7_B",   id: NA,       group: Pwm    }, // GP15

        // I2C
        Def { alias: "I2C0_SDA", id: Gpio(2),  group: I2c    }, // GP0, GP4, GP8, GP12, GP16, GP20, GP28
        Def { alias: "I2C0_SCL", id: NA,       group: I2c    }, // GP1, GP5, GP9, GP13, GP17, GP21
        Def { alias: "I2C1_SDA", id: NA,       group: I2c    }, // GP2, GP6, GP10, GP14, GP18, GP22, GP26
        Def { alias: "I2C1_SCL", id: NA,       group: I2c    }, // GP3, GP7, GP11, GP15, GP19, GP27

        // SPI
        Def { alias: "SPI0_RX",  id: Gpio(4),  group: Spi    }, // GP0, GP4, GP16, GP20
        Def { alias: "SPI0_TX",  id: NA,       group: Spi    }, // GP3, GP19
        Def { alias: "SPI0_SCK", id: NA,       group: Spi    }, // GP2, GP18, GP22
        Def { alias: "SPI0_CSN", id: NA,       group: Spi    }, // GP1, GP5, GP17, GP21

        Def { alias: "SPI1_RX",  id: NA,       group: Spi    }, // GP8, GP12, GP28
        Def { alias: "SPI1_TX",  id: NA,       group: Spi    }, // GP7, GP11, GP15, GP27
        Def { alias: "SPI1_SCK", id: NA,       group: Spi    }, // GP6, GP10, GP14, GP26
        Def { alias: "SPI1_CSN", id: NA,       group: Spi    }, // GP9, GP13

        // UART
        Def { alias: "UART0_TX",  id: Gpio(5),  group: Uart  }, // GP0, GP12, GP16, GP28
        Def { alias: "UART0_CTS", id: NA,       group: Uart  }, // GP2, GP14, GP18
        Def { alias: "UART0_RX",  id: NA,       group: Uart  }, // GP1, GP13, GP17
        Def { alias: "UART0_RTS", id: NA,       group: Uart  }, // GP3, GP15, GP19
        
        Def { alias: "UART1_TX",  id: NA,       group: Uart  }, // GP4, GP8, GP20
        Def { alias: "UART1_RX",  id: NA,       group: Uart  }, // GP5, GP9, GP21
        Def { alias: "UART1_CTS", id: NA,       group: Uart  }, // GP6, GP10, GP22, GP26
        Def { alias: "UART1_RTS", id: NA,       group: Uart  }, // GP7, GP11, GP27

        // Inputs - Add your own aliases
        Def { alias: "IN_A",     id: Gpio(9),  group: Inputs  },
        Def { alias: "IN_B",     id: Gpio(20), group: Inputs  },
        Def { alias: "IN_C",     id: Gpio(22), group: Inputs  },
        Def { alias: "BUTTON",   id: Gpio(23), group: Inputs  },

        // Ouputs 
        Def { alias: "OUT_A",    id: Gpio(0),  group: Outputs },
        Def { alias: "OUT_B",    id: Gpio(1),  group: Outputs },
        Def { alias: "OUT_C",    id: Gpio(3),  group: Outputs },
        Def { alias: "LED",      id: Gpio(25), group: Outputs },
    ]
};

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::new(PIN_DEFINITION));

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
      .ok_or(Error::GpioNotFound)
  }

  /// Gets the string alias associated with a given pin GPIO number (`u8`).
  pub fn get_alias(&self, id: u8) -> Result<&'static str> {
    self
      .pins
      .iter()
      .find(|def| def.id == id)
      .map(|def| def.alias)
      .ok_or(Error::AliasNotFound)
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

/// The functional group a pin belongs to.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Group {
  Adc,
  Pwm,
  I2c,
  Spi,
  Uart,
  Inputs,
  Outputs,
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
  InvalidPin,
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// —————————————————————————————————————————————————————————————————————————————————————————————————

impl fmt::Display for Group {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Group::Adc => write!(f, "ADC"),
      Group::Pwm => write!(f, "PWM"),
      Group::I2c => write!(f, "I2C"),
      Group::Spi => write!(f, "SPI"),
      Group::Uart => write!(f, "UART"),
      Group::Inputs => write!(f, "Input"),
      Group::Outputs => write!(f, "Output"),
    }
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
    $crate::config::CONFIG.get_gpio(stringify!($alias)).unwrap()
  };
}
