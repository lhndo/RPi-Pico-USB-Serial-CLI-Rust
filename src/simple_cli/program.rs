//! Main CLI program logic
/// ————————————————————————————————————————————————————————————————————————————————————————————————
///                                            Program
/// ————————————————————————————————————————————————————————————————————————————————————————————————
use super::*;

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
  // ———————————————————————————————————————————— New ———————————————————————————————————————————————

  pub fn new() -> Self {
    let command_buf = FifoBuffer::<CMD_BUFF_SIZE>::new();
    let command_read = false;

    Self { command_buf, command_read }
  }

  // ———————————————————————————————————————————— Init ——————————————————————————————————————————————

  pub fn init(&mut self, device: &mut Device) {
    // Blocking wait until we receive a serial monitor connection
    let led = device.outputs.get_pin(LED).unwrap();

    // While we don't have a serial monitor connected we keep trying
    while SERIAL.get_drt() != Ok(true) {
      led.toggle().unwrap();
      SERIAL.poll_usb();
      DELAY.ms(80);
    }

    // Blink leds four times to notify
    // This also warms up the USB device, which otherwise will skip the print msg below
    for _ in 0..4 {
      led.set_low().unwrap();
      device.timer.delay_ms(200);
      led.set_high().unwrap();
      device.timer.delay_ms(200);
    }

    SERIAL.set_connected(true);

    // Displaying last panic msg
    if let Some(msg) = panic_persist::get_panic_message_bytes() {
      println!("\n========= PANIC ===========");
      println!("{}", msg.as_str());
    }

    println!("\n========= HELLO =========== ");
    let time = device.timer.get_counter().ticks();
    println!("Current timer ticks: {} (T: {})", time, device.timer.print_time());
    println!("Frequency: {}hz", SYS_CLK_HZ);
    println!("Type \"help\" for the command lists\n");
  }

  // ———————————————————————————————————————————— Run ———————————————————————————————————————————————

  pub fn run(&mut self, device: &mut Device) {
    let led = device.outputs.get_pin(LED).unwrap();
    led.set_high().unwrap();

    let mut cli = Cli::new(&CMDS);

    loop {
      if SERIAL.is_connected() != Ok(true) {
        continue;
      }

      SERIAL.poll_usb();

      // Read command
      if !self.command_read {
        // Print Device Status
        let temp_adc_raw: u16 = device.acds.read_channel(TEMP_SENSE_CHN).unwrap_or(0);
        let vsys_adc_raw: u16 = device.acds.read_channel(3).unwrap_or(0);
        let sys_temp = 27.0 - (temp_adc_raw.to_voltage() - 0.706) / 0.001721;

        print!("\n| Temp: {:.1}C Voltage: {:.2}V | ", sys_temp, vsys_adc_raw.to_voltage());
        print!("Enter Command >>> \n");

        // Blocking wait for command
        self.command_buf.clear();
        let len = SERIAL.read_line(self.command_buf.receive_buffer()).unwrap_or(0);
        if len > 0 {
          self.command_buf.advance(len);
          self.command_read = true;
          println!("\n>> Received Command: (T: {}) ", device.timer.print_time());
        }
      }

      // Execute command
      if self.command_read {
        let input = self.command_buf.data().as_str();
        println!(">> '{}' \n", input);
        cli.execute(input, device).unwrap_or_else(|e| println!("Err: {}", e));

        // Cleanup
        self.command_buf.clear();
        self.command_read = false; // Done, accepting new cmds
        print!("\n========= DONE =========== (T: {}) \n", device.timer.print_time());
      }

      let led = device.outputs.get_pin(LED).unwrap();
      for _ in 0..3 {
        led.set_low().unwrap();
        device.timer.delay_ms(50);
        led.set_high().unwrap();
        device.timer.delay_ms(50);
      }
    }
  }
}
