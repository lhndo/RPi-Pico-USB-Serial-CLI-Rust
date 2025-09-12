//! Main CLI program logic
/// ———————————————————————————————————————————————————————————————————————————————————————————————
///                                            Program
/// ———————————————————————————————————————————————————————————————————————————————————————————————
use super::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

static CMD_BUFF_SIZE: usize = 192;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Program Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Program {
  pub command_buf:  FifoBuffer<u8, CMD_BUFF_SIZE>,
  pub command_read: bool,
}

impl Program {
  // ———————————————————————————————————————————— New ——————————————————————————————————————————————

  pub fn new() -> Self {
    let command_buf = FifoBuffer::new();
    let command_read = false;

    Self { command_buf, command_read }
  }

  // ———————————————————————————————————————————— Init —————————————————————————————————————————————

  pub fn init(&mut self, device: &mut Device) {
    let led = device.outputs.get_pin(PinID::LED).unwrap();

    // While we don't have a serial monitor connection we keep polling
    while !SERIAL.is_connected() {
      led.toggle().unwrap();
      device.timer.delay_ms(80);
    }

    #[cfg(feature = "defmt")]
    info!("USB Serial Monitor: Connected!");

    // Blink leds four times to notify
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

    println!("\n========= HELLO =========== ");
    let time = device.timer.get_counter().ticks();
    println!("Current timer ticks: {} (T: {})", time, device.timer.print_time());
    println!("Frequency: {}hz", SYS_CLK_HZ.load(Ordering::Relaxed));
    println!("Type \"help\" for the command lists\n");
  }

  // ———————————————————————————————————————————— Run ———————————————————————————————————————————————

  pub fn run(&mut self, device: &mut Device) {
    let mut cli = Cli::new(&CMDS);

    let led = device.outputs.get_pin(PinID::LED).unwrap();
    led.set_high().unwrap();

    loop {
      // While we don't have a serial monitor connection we keep polling
      let led = device.outputs.get_pin(PinID::LED).unwrap();
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
        let sys_temp = 27.0 - (temp_adc_raw.to_voltage() - 0.706) / 0.001721;

        print!("\n| Temp: {:.1}C A3: {:.2}V | ", sys_temp, vsys_adc_raw.to_voltage());
        print!("Enter Command >>> \n");

        // Blocking wait for command
        self.command_buf.clear();
        match SERIAL.read_line_blocking(self.command_buf.receive_buffer()) {
          Ok(len) => {
            self.command_buf.advance(len);
            self.command_read = true;
            println!("\n>> Received Command: (T: {}) ", device.timer.print_time());
          },
          Err(e) => {
            println!("\nErr: {:?} \n", e);
            continue;
          },
        }
      }

      // Execute command
      if self.command_read {
        let input = self.command_buf.data().as_str().unwrap();
        println!(">> '{}' \n", input);
        cli.execute(input, device).unwrap_or_else(|e| println!("Err: {}", e));

        // Cleanup
        self.command_buf.clear();
        self.command_read = false; // Done, accepting new cmds
        print!("\n========= DONE =========== (T: {}) \n", device.timer.print_time());
      }

      let led = device.outputs.get_pin(PinID::LED).unwrap();
      for _ in 0..3 {
        led.set_low().unwrap();
        device.timer.delay_ms(50);
        led.set_high().unwrap();
        device.timer.delay_ms(50);
      }
    }
  }
}
