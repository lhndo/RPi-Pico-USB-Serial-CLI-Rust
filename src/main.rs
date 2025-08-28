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

use device::Device;

use panic_persist as _;

static RUN_STANDALONE: bool = false;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Main
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[rp_pico::entry]
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
