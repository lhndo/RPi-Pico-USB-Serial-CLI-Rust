//! Main CLI program logic
/// ————————————————————————————————————————————————————————————————————————————————————————————————
///                                            Program
/// ————————————————————————————————————————————————————————————————————————————————————————————————
use super::cli_commands::CMDS;
use super::*;

use crate::prelude::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

static CMD_BUFF_SIZE: usize = 128;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Program Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Program {
  pub command_buf:  FifoBuffer<CMD_BUFF_SIZE>,
  pub command_read: bool,
}


impl Program {
  // ————————————————————————————————————————————————————————————————————————————————————————————————
  //                                              New
  // ————————————————————————————————————————————————————————————————————————————————————————————————

  pub fn new() -> Self {
    let command_buf = FifoBuffer::<CMD_BUFF_SIZE>::new();
    let command_read = false;


    Self { command_buf, command_read }
  }


  // ————————————————————————————————————————————————————————————————————————————————————————————————
  //                                             Init
  // ————————————————————————————————————————————————————————————————————————————————————————————————

  pub fn init(&mut self, device: &mut Device) {
    let mut said_hello = false;

    while !said_hello {
      SERIAL.poll_usb();
      DELAY.us(10);

      if device.timer.get_counter().ticks() >= 2_000_000 {
        said_hello = true;
        print!("\nHello!\n");

        let time = device.timer.get_counter().ticks();
        print!("Current timer ticks: {} (T: {})", time, device.timer.print_time());

        device.outputs.led.set_high().unwrap();
      }
    }
  }


  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                             Run
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  pub fn run(&mut self, device: &mut Device) {
    device.outputs.led.set_high().unwrap();

    let mut cli = Cli::new(&CMDS);

    loop {
      // ————————————————————————————————————— Read Command ———————————————————————————————————————

      SERIAL.poll_usb();

      if !self.command_read {
        // Print Device Status
        let temp_adc_raw: u16 = device.hal_adc.read(&mut device.acds.acd4_temp_sense).unwrap();
        let vsys_adc_raw: u16 = device.hal_adc.read(&mut device.acds.adc3).unwrap();
        let sys_temp = 27.0 - (temp_adc_raw.to_voltage() - 0.706) / 0.001721;

        print!("\n| Temp: {:.1}C Voltage: {:.2}V | ", sys_temp, vsys_adc_raw.to_voltage());
        print!("Enter Command >>> \n");

        // Blocking wait for command
        self.command_buf.clear();
        let len = SERIAL.read_line(self.command_buf.receive_buffer());
        if len > 0 {
          self.command_buf.advance(len);
          self.command_read = true;
          println!("\n>> Received Command: (T: {}) ", device.timer.print_time());
        }
      }

      // ——————————————————————————————————————— Process ——————————————————————————————————————————

      if self.command_read {
        let input = self.command_buf.data().as_str();
        println!(">> '{}' \n", input);
        cli.execute(input, device).unwrap_or_else(|e| println!("Err: {}", e));


        // ——————————————————————————————————————— End ——————————————————————————————————————————

        self.command_buf.clear();
        self.command_read = false; // Done, accepting new cmds
        print!("\n========= DONE =========== (T: {}) \n", device.timer.print_time());
      }

      device.outputs.led.toggle().unwrap();
    }
  }
}
