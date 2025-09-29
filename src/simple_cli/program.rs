//! Main CLI entry point and program logic
//!
//! To be used in main program loop
//!
//! ```no_run
//! fn main() -> ! {
//!   let mut program = simple_cli::program::Program::new();
//!   let command_list = simple_cli::commands::build_commands();
//!   program.init(&mut device);
//!   program.run(&mut device, command_list);
//! }
//! ```

use super::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

const CMD_BUFF_SIZE: usize = 192;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Program
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Program {
  pub command_buf:  FifoBuffer<CMD_BUFF_SIZE>,
  pub command_read: bool,
}

impl Program {
  pub fn new() -> Self {
    let command_buf = FifoBuffer::new();
    let command_read = false;

    Self { command_buf, command_read }
  }

  // —————————————————————————————————————————————————————————————————————————————————————————————————
  //                                              Init
  // —————————————————————————————————————————————————————————————————————————————————————————————————

  pub fn init(&mut self, device: &mut Device) {
    let led = device.outputs.get_by_id(PinID::LED).unwrap();

    // While we don't have a serial monitor connection we keep polling and bliking led for status
    while !SERIAL.is_connected() {
      led.toggle().unwrap();
      device.timer.delay_ms(80);
    }

    #[cfg(feature = "defmt")]
    info!("USB Serial Monitor: Connected!");

    // Blink leds four times to notify connected
    for _ in 0..4 {
      led.set_low().unwrap();
      device.timer.delay_ms(200);
      led.set_high().unwrap();
      device.timer.delay_ms(200);
    }

    // Displaying last panic msg
    #[cfg(feature = "panic-persist")]
    if let Some(msg) = panic_persist::get_panic_message_bytes() {
      println!("\n========= PANIC ===========");
      if let Ok(msg) = msg.as_str() {
        println!("{}", msg);
      }
    }

    // Print greeting msg
    println!("\n========= HELLO =========== ");
    let time_ticks = device.timer.get_counter().ticks();
    println!("Current timer ticks: {time_ticks} (T: {})", device.timer.print_time());
    println!("Frequency: {}hz", SYS_CLK_HZ.load(Ordering::Relaxed));
    println!("Type \"help\" for the command lists\n");
  }

  // —————————————————————————————————————————————————————————————————————————————————————————————————
  //                                               Run
  // —————————————————————————————————————————————————————————————————————————————————————————————————

  pub fn run(&mut self, device: &mut Device, commands: CommandList) {
    let mut cli = Cli::new(commands);

    let led = device.outputs.get_by_id(PinID::LED).unwrap();
    led.set_high().unwrap();

    loop {
      // While we don't have a serial monitor connection we keep polling
      let led = device.outputs.get_by_id(PinID::LED).unwrap();
      while !SERIAL.is_connected() {
        led.toggle().unwrap();
        device.timer.delay_ms(80);
      }

      led.set_high().unwrap();

      // Read command
      if !self.command_read {
        // Print Device Status
        let temp_adc_raw: u16 = device.adcs.read_channel(TEMP_SENSE_CHN).unwrap_or(0);
        let vsys_adc_raw: u16 = device.adcs.read_channel(3).unwrap_or(0);
        let sys_temp = 27.0 - (temp_adc_raw.to_voltage() - 0.706) / 0.001721; // RP2040 internal temp sensor calibration

        print!("\n| Temp: {:.1}C A3: {:.2}V | ", sys_temp, vsys_adc_raw.to_voltage());
        print!("Enter Command >>> \n");

        // Blocking wait for command
        self.command_buf.clear();
        match SERIAL.read_line_blocking(self.command_buf.receive_buffer()) {
          Ok(len) => {
            self.command_buf.advance(len);
            self.command_read = true;
            println!("\n>> Received Command:");
          },
          Err(e) => {
            println!("\nErr: {:?} \n", e);
            continue;
          },
        }
      }

      // Execute command
      if self.command_read {
        let input = self.command_buf.get_data().as_str().unwrap();
        println!(">> '{}' ", input);

        println!("\n======== RUNNING ========= (T: {}) \n", device.timer.print_time());
        let exec_time = device.timer.get_counter();

        cli.execute(input, device).unwrap_or_else(|e| println!("Err: {}", e));

        let exec_time = device
          .timer
          .get_counter()
          .checked_duration_since(exec_time)
          .unwrap()
          .to_micros();

        // Cleanup
        self.command_buf.clear();
        self.command_read = false; // Done, accepting new cmds

        print!(
          "\n===== DONE in {time:.3}ms ===== (T: {}) \n",
          device.timer.print_time(),
          time = exec_time as f32 / 1000.0
        );
      }

      // Signal End
      let led = device.outputs.get_by_id(PinID::LED).unwrap();
      for _ in 0..3 {
        led.set_low().unwrap();
        device.timer.delay_ms(50);
        led.set_high().unwrap();
        device.timer.delay_ms(50);
      }
    }
  }
}
