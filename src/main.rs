// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     RP Pico Serial USB CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![no_std]
#![no_main]

mod delay;
mod device;
mod fifo_buffer;
mod prelude;
mod serial_io;
mod simple_cli;

use device::Device;

use rp_pico as bsp;
use rp2040_panic_usb_boot as _;

static RUN_STANDALONE: bool = false;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Main
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[bsp::entry]
fn main() -> ! {
  let mut device = Device::new();

  if !RUN_STANDALONE {
    let mut program = simple_cli::program::Program::new();

    program.init(&mut device);
    program.run(&mut device);
  }

  loop {
    device::device_reset_to_usb();
  }
}
