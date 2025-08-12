// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                     RP Pico Serial USB CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![no_std]
#![no_main]

mod delay;
mod device;
mod fifo_buffer;
mod prelude;
mod program;
mod serial_io;
mod simple_cli;

use device::Device;
use program::Program;

use rp_pico as bsp;
use rp2040_panic_usb_boot as _;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Main
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[bsp::entry]
fn main() -> ! {
  let mut device = Device::new();

  if true {
    let mut program = Program::new();

    program.init(&mut device);
    program.run(&mut device);
  }

  loop {
    device::device_reset_to_usb();
  }
}
