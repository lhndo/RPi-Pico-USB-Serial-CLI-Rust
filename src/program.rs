//! Main CLI entry point and program logic
//!
//! To be used in main program loop
//!
//! Example
//!
//! ```no_run
//! fn main() -> ! {
//!     let command_list = simple_cli::commands::build_command_list();
//!     let mut program = program::Program::new();
//!     program.run(&mut device, command_list);
//! }
//! ```

use crate::cli::CommandList;
use crate::cli::SimpleCli;
use crate::prelude::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

const CMD_BUFF_SIZE: usize = 192;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Program
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Program {}

impl Program {
    pub fn new() -> Self {
        Self {}
    }

    // —————————————————————————————————————————————————————————————————————————————————————————————————
    //                                               Run
    // —————————————————————————————————————————————————————————————————————————————————————————————————

    pub fn run(&mut self, device: &mut Device, commands: CommandList) {
        let mut command_buf: FifoBuffer<CMD_BUFF_SIZE> = FifoBuffer::new();
        let mut command_read = false;
        let mut cli = SimpleCli::new(commands);

        loop {
            // —————————————————————————————————— Acquire Connection —————————————————————————————————————

            // While we don't have a serial monitor connection we keep polling
            if !SERIAL.is_connected() {
                self.get_connection(device);
                self.greet(device);
            }

            let led = device.outputs.get(gpio!(LED)).unwrap();
            led.set_high().unwrap();

            // ————————————————————————————————————— Read command ————————————————————————————————————————
            if !command_read {
                // Print Device Status
                let temp_adc_raw: u16 = device.adcs.read(TEMP_SENSE_CHN).unwrap_or(0);
                let vsys_adc_raw: u16 = device.adcs.read(3).unwrap_or(0);
                let sys_temp = 27.0 - (temp_adc_raw.to_voltage() - 0.706) / 0.001721; // RP2040 temp sensor calibration

                println!(
                    "\n| Temp: {:.1}C | A3: {:.2}V | T: {} |",
                    sys_temp,
                    vsys_adc_raw.to_voltage(),
                    device.timer.print_time()
                );
                print!("Enter Command: \n>>> ");

                // Blocking - Waiting for a command
                command_buf.clear();
                match SERIAL.read_line_blocking(command_buf.receive_buffer()) {
                    Ok(len) => {
                        command_buf.advance(len);
                        command_read = true;
                        let data = command_buf.get_data().as_str().unwrap();
                        println!("{}", data);
                    }
                    Err(e) => {
                        println!("\nErr: {:?} \n", e);
                        continue;
                    }
                }
            }

            // ———————————————————————————————————— Execute command ——————————————————————————————————————

            if command_read {
                let input = command_buf.get_data().as_str().unwrap();
                let cmd_name = input.split_ascii_whitespace().next().unwrap_or("help");

                println!("\n========= RUNNING: {cmd_name} =========\n");

                // Time benchmark start
                let exec_time = device.timer.get_counter();

                cli.execute(input, device).unwrap_or_else(|e| println!("Err: {}", e));

                // Time benchmark end
                let exec_time = device
                    .timer
                    .get_counter()
                    .checked_duration_since(exec_time)
                    .unwrap()
                    .to_micros();

                // Cleanup
                command_buf.clear();
                command_read = false; // Done, accepting new cmds

                println!(
                    "\n========= DONE in {time:.3}ms =========\n",
                    time = exec_time as f32 / 1000.0
                );
            }

            // ————————————————————————————————— Signal Execution End ————————————————————————————————————

            let led = device.outputs.get(gpio!(LED)).unwrap();
            for _ in 0..3 {
                led.set_low().unwrap();
                device.timer.delay_ms(50);
                led.set_high().unwrap();
                device.timer.delay_ms(50);
            }
        }
    }

    // —————————————————————————————————————————————————————————————————————————————————————————————————
    //                                           Get Connection
    // —————————————————————————————————————————————————————————————————————————————————————————————————

    /// Blocking function until connection is acquired
    fn get_connection(&mut self, device: &mut Device) {
        let led = device.outputs.get(gpio!(LED)).unwrap();

        // While we don't have a serial monitor connection we keep polling and bliking led for status
        while !SERIAL.is_connected() {
            led.toggle().unwrap();
            device.timer.delay_ms(80);
        }
        info!("USB Serial Monitor: Connected!");
    }

    // —————————————————————————————————————————————————————————————————————————————————————————————————
    //                                              Greet
    // —————————————————————————————————————————————————————————————————————————————————————————————————

    fn greet(&mut self, device: &mut Device) {
        let led = device.outputs.get(gpio!(LED)).unwrap();

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
        let time_ticks = device.timer.get_counter().ticks();
        println!("\n========= HELLO =========== ");
        println!("Current timer ticks: {time_ticks} (T: {})", device.timer.print_time());
        println!("Frequency: {}hz", SYS_CLK_HZ.load(Ordering::Relaxed));
        println!("Type \"help\" for the command lists\n");
    }
}
