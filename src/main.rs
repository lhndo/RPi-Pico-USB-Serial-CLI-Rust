// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     RP Pico Serial USB CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![no_std]
#![no_main]

mod system;
mod utils;

mod cli;
mod main_core1;
mod pin_config;
mod prelude;
mod program;
mod state;

// ———————————————————————————————————— Debug dfmt features ——————————————————————————————————————
#[cfg(feature = "defmt")]
use defmt_rtt as _;

#[allow(unused_imports)]
#[cfg(feature = "defmt")]
use defmt::{debug, error, info, trace, warn};

// ——————————————————————————————— Panic handler select features ——————————————————————————————————
#[cfg(feature = "panic-probe")]
extern crate panic_probe;

#[cfg(feature = "panic-usb")]
extern crate rp2040_panic_usb_boot;

#[cfg(feature = "panic-persist")]
extern crate panic_persist;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

const RUN_STANDALONE: bool = false;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Main
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[rp2040_hal::entry]
fn main() -> ! {
  //

  info!("Alive! {} : v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

  let mut device = system::device::Device::new();

  if !RUN_STANDALONE {
    let command_list = cli::commands::build();
    let mut program = program::Program::new();
    program.run(&mut device, command_list);
  }

  loop {
    system::device::device_reset_to_usb();
  }
}
