// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Program
// ————————————————————————————————————————————————————————————————————————————————————————————————
// Contains the main program logic


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
      DELAY.delay_us(10);

      if device.timer.get_counter().ticks() >= 2_000_000 {
        said_hello = true;
        print!("Hello, World!\n");

        let time = device.timer.get_counter().ticks();
        print!("Current timer ticks: {} (T: {})", time, device.timer.print_time());

        device.pins.led.set_high().unwrap();
      }
    }
  }


  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                             Run
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  pub fn run(&mut self, device: &mut Device) {
    device.pins.led.set_high().unwrap();

    loop {
      // ————————————————————————————————————— Read Command ———————————————————————————————————————

      SERIAL.poll_usb();

      if !self.command_read {
        // Print Device Status
        let temp_adc: u16 = device.adc.read(&mut device.pins.temp_sense).unwrap();
        let v_adc: u16 = device.adc.read(&mut device.pins.adc_pin).unwrap();
        let temp = 27.0 - (temp_adc.to_voltage() - 0.706) / 0.001721;

        print!("\n| Temp: {:.1}C Voltage: {:.2}V |\n", temp, v_adc.to_voltage());
        print!("Enter Command: \n");

        // Blocking wait for command
        self.command_buf.clear();
        let len = SERIAL.read_line(self.command_buf.receive_buffer());
        if len > 0 {
          self.command_buf.advance(len);
          self.command_read = true;

          print!("\n>> Command Received. (T: {}) \n>> ", device.timer.print_time());
          println!("{}", self.command_buf.data().as_str());
        }
      }

      // ——————————————————————————————————————— Process ——————————————————————————————————————————

      if self.command_read {
        // Quick draft commands
        if self.command_buf.contains_str("reset").is_some() {
          print!("\nResetting...\n");
          DELAY.delay_ms(1500); // Waiting for reset msg to appear
          device_reset();
        }
        if self.command_buf.contains_str("flash").is_some() {
          print!("\nRestarting in USB Flash mode!...\n");
          device_reset_to_usb();
        }


        println!("\nTransferring buffer ...");
        print!("Echo: \n");
        print!("{}", self.command_buf.data().as_str());

        // ——————————————————————————————————————— Write ——————————————————————————————————————————

        if self.command_read {
          SERIAL.flush_write();
          self.command_buf.clear();
          self.command_read = false; // Done, accepting new cmds
          print!("\n==================== (T: {}) \n", device.timer.print_time());
        }
      }

      device.pins.led.toggle().unwrap();
    }
  }
}
