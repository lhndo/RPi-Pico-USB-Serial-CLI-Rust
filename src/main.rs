// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     RP Pico Serial USB CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![no_std]
#![no_main]

mod adcs;
mod delay;
mod device;
mod fifo_buffer;
mod gpios;
mod prelude;
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
#[cfg(feature = "defmt")]
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

#[rp_pico::entry]
fn main() -> ! {
  //
  #[cfg(feature = "defmt")]
  info!("Alive");

  let mut device = device::Device::new();

  if !RUN_STANDALONE {
    let mut program = simple_cli::program::Program::new();
    program.init(&mut device);
    program.run(&mut device);
  }

  loop {
    device::device_reset_to_usb();
  }
}
