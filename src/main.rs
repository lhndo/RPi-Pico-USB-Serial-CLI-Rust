// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     RP Pico Serial USB CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![no_std]
#![no_main]

mod adcs;
mod config;
mod delay;
mod device;
mod fifo_buffer;
mod gpios;
mod log;
mod prelude;
mod program;
mod pwms;
mod serial_io;
mod simple_cli;
mod state;
mod tasklet;

// ———————————————————————————————————— Debug dfmt features ——————————————————————————————————————
#[cfg(feature = "defmt")]
use defmt_rtt as _;

#[allow(unused_imports)]
#[cfg(feature = "defmt")]
use defmt::{debug, error, info, warn};

// ——————————————————————————————— Panic handler select features ——————————————————————————————————
#[cfg(feature = "panic-probe")]
extern crate panic_probe;

#[cfg(feature = "panic-usb")]
extern crate rp2040_panic_usb_boot;

#[cfg(feature = "panic-persist")]
extern crate panic_persist;

// —————————————————————————————————————————— Globals —————————————————————————————————————————————

const RUN_STANDALONE: bool = false;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Main
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[rp2040_hal::entry]
fn main() -> ! {
  //

  info!("Alive! {} : v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

  let mut device = device::Device::new();

  if !RUN_STANDALONE {
    let command_list = simple_cli::commands::build_command_list();
    let mut program = program::Program::new();
    program.run(&mut device, command_list);
  }

  loop {
    device::device_reset_to_usb();
  }
}
