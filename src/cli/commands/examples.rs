//! Example Commands
// Register new commands in commands.rs > Command List Builder

use super::*;
use crate::prelude::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Example
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_example_cmd() -> Command {
  Command {
    name: "example",
    desc: "Prints example args",
    help: "example <arg(float)> [opt=0(u8)] [on=false(bool)] [path=\"\"(string)] [help]",
    func: example_cmd,
  }
}

pub fn example_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let arg: f32 = args.get_parsed_param("arg")?; // required argument
  let opt: u8 = args.get_parsed_param("opt").unwrap_or(0); // argument with default
  let on: bool = args.get_parsed_param("on").unwrap_or(false);
  let path: &str = args.get_str_param("path").unwrap_or("");

  println!("---- Running 'Example' ---- \n");

  println!("arg = {arg}");
  println!("opt = {opt}");
  println!("on = {on}");
  println!("path = {path}");

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Blink
// —————————————————————————————————————————————————————————————————————————————————————————————————

// Blink example
// ex: blink times=4

pub fn build_blink_cmd() -> Command {
  Command {
    name: "blink",
    desc: "Blinks Onboard Led",
    help: "blink [times=10] [interval=200(ms)] [help]",
    func: blink_cmd,
  }
}

pub fn blink_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let times: u16 = args.get_parsed_param("times").unwrap_or(10); // 10 default
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200); // 200ms default

  println!("---- Blinking Led! ----\n");
  let led = device.outputs.get(gpio!(LED)).unwrap();

  // Non blocking timer based task
  let mut ledtask = Tasklet::new(interval as u32, times * 2, &device.timer);

  let mut blink = 1;

  while !ledtask.is_exhausted() {
    if ledtask.is_ready() {
      led.toggle().unwrap();

      if led.is_set_high().unwrap() {
        print!("Blink {} | ", blink);
        blink += 1;
      }
    }
  }

  // Non tasklet implementation example:
  //
  // for n in 1..=times {
  //   print!("Blink {} | ", n);
  //   led.set_high().unwrap();
  //   device.timer.delay_ms(interval);
  //   led.set_low().unwrap();
  //   device.timer.delay_ms(interval);
  // }

  println!();
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Blink Multicore
// —————————————————————————————————————————————————————————————————————————————————————————————————

// Blink example
// ex: blink times=4

pub fn build_blink_multicore_cmd() -> Command {
  Command {
    name: "blink_multicore",
    desc: "Blinks Onboard Led using by passing an event to Core1",
    help: "blink [times=10] [interval=200(ms)] [help]",
    func: blink_multicore_cmd,
  }
}

pub fn blink_multicore_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let times: u16 = args.get_parsed_param("times").unwrap_or(10); // 10 default
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200); // 200ms default

  println!("---- Blinking Led using Core1! ----\n");

  CORE1_QUEUE
    .enqueue(Event::Blink {
      times:    times,
      interval: interval,
    })
    .ok();

  // We wait since we don't have a done callback implemented
  for blink in 1..=times {
    print!("Blink {} | ", blink);
    device.timer.delay_ms(interval * 2);
  }

  println!();
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Servo
// —————————————————————————————————————————————————————————————————————————————————————————————————
// Angle Controlled RC Servo
// ex: servo us=1200
// ex: servo sweep max_us=1800

pub fn build_servo_cmd() -> Command {
  Command {
    name: "servo",
    desc: "Set Servo PWM on GPIO 8",
    help: "servo [alias=PWM4_A(str)] / [gpio=..(u8)] [us=1500(us)] [pause=1000(ms)]\n      \
           [sweep] [max_us=2000(us)] [help]",
    func: servo_cmd,
  }
}

pub fn servo_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  const DEFAULT_PIN: &str = "PWM4_A";

  // Getting Alias or GPIO input ---------
  let alias = args.get_str_param("alias").unwrap_or(DEFAULT_PIN);
  let gpio = args.get_parsed_param::<u8>("gpio").ok();

  let (gpio, alias) = CONFIG.get_gpio_alias_pair(gpio, Some(alias))?;
  // -------------------------------------

  let us: u16 = args.get_parsed_param("us").unwrap_or(1500); //  1500 us default
  let pause: u32 = args.get_parsed_param("pause").unwrap_or(1000); // 1s default
  let max_us: u16 = args.get_parsed_param("max_us").unwrap_or(2000); //  2000 us default
  let sweep: bool = args.contains_param("sweep");

  // Validating pwm pin
  let (pwm_id, channel) = device.pwms.get_pwm_slice_id_by_gpio(gpio)?;

  println!("---- Servo ----");
  println!("Servo: GPIO {gpio} - {alias} | pwm: {pwm_id}, channel: {channel}");

  // —————————————————————————————————————————— Program ————————————————————————————————————————————
  const FREQ: u32 = 50;
  println!("\nSetting: Duty: {}us, Freq: {}", us, FREQ);

  // Initializing pwm slice frequency
  with_pwm_slice!(&mut device.pwms, pwm_id, |pwm_slice| {
    pwm_slice.set_freq(FREQ);
    pwm_slice.enable();
  });

  // Set us duty
  let mut servo_pin = device.pwms.get_channel_by_gpio(gpio).unwrap();
  servo_pin.set_duty_cycle_us(us, FREQ);
  device.timer.delay_ms(pause);

  // Sweep Mode
  if sweep {
    // Sweeping from us to max_us
    println!("Sweeping between: {us}us - {max_us}us in {}ms \n ...", pause * 4);
    let sweep_time = (pause * 2) as f32;
    let start_time = device.timer.now();

    // PWM duty based on elapsed time and phase
    loop {
      let elapsed_ms = (device.timer.now() - start_time).to_millis() as f32;

      // Calculate which phase of the sweep we're in (0-1)
      let phase = (elapsed_ms / sweep_time) as u32;
      // Calculate progress within current phase (0.0 to 1.0)
      let progress = (elapsed_ms - (phase as f32 * sweep_time)) / sweep_time;

      let target_us = match phase {
        0 => us + ((max_us - us) as f32 * progress) as u16, //     Min to Max
        1 => max_us - ((max_us - us) as f32 * progress) as u16, // Max to Min
        _ => break,                                         //     Done
      };

      servo_pin.set_duty_cycle_us(target_us, FREQ);
    }

    println!("Sweeping complete");
  }

  // Off
  servo_pin.set_duty_cycle_fully_off().unwrap();
  println!("Done!");
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Test GPIO
// —————————————————————————————————————————————————————————————————————————————————————————————————
// Toggle an output pin based on an input pin

pub fn build_test_gpio_cmd() -> Command {
  Command {
    name: "test_gpio",
    desc: "Sets output HIGH when input is LOW",
    help: "test_gpio [input=IN_A(str)] [output=OUT_A(str)] [help] \nInterrupt with char \"~\" ",
    func: test_gpio_cmd,
  }
}

pub fn test_gpio_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  const DEFAULT_INPUT: &str = "IN_A";
  const DEFAULT_OUTPUT: &str = "OUT_A";

  let input = args.get_str_param("input").unwrap_or(DEFAULT_INPUT);
  let output = args.get_str_param("output").unwrap_or(DEFAULT_OUTPUT);

  let gpio_input = CONFIG.get_gpio(input)?;
  let gpio_output = CONFIG.get_gpio(output)?;

  println!("---- Testing GPIO ----");
  println!("Input: GPIO {gpio_input} - {input} >> Output: GPIO {gpio_output} {output}");
  println!("\nSend '~' to exit\n");

  let input = device.inputs.get(gpio_input).unwrap();
  let output = device.outputs.get(gpio_output).unwrap();

  // Loop
  SERIAL.clear_interrupt_cmd();
  while !SERIAL.interrupt_cmd_triggered() {
    if input.is_low().unwrap() {
      output.set_high().unwrap();
    }
    else {
      output.set_low().unwrap();
    }
  }

  println!("Done!");
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Test Analog
// —————————————————————————————————————————————————————————————————————————————————————————————————
// Control the PWM duty cycle of the output pin using an ADC input pin (potentiometer)
// Can be used to control a servo with a potentiometer using the min max us limits (try 500-2500)

pub fn build_test_analog_cmd() -> Command {
  Command {
    name: "test_analog",
    desc: "Voltage controlled PWM Duty Cycle",
    help:
      "test_analog [input=ADC0(str)] [output=PWM4_A(str)] [min_us=..(us)] [max_us=..(us)]\n      \
       [help] \nInterrupt with char \"~\" ",
    func: test_analog_cmd,
  }
}

pub fn test_analog_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  const DEFAULT_INPUT: &str = "ADC0";
  const DEFAULT_OUTPUT: &str = "PWM4_A";

  let input = args.get_str_param("input").unwrap_or(DEFAULT_INPUT);
  let output = args.get_str_param("output").unwrap_or(DEFAULT_OUTPUT);

  let gpio_input = CONFIG.get_gpio(input)?;
  let gpio_output = CONFIG.get_gpio(output)?;

  let min_us = args.get_parsed_param("min_us").unwrap_or(0);
  let max_us = args.get_parsed_param("max_us").unwrap_or(0);

  println!("---- Testing Analog Input ----");
  println!("Input: GPIO {gpio_input} - {input} >> Output: GPIO {gpio_output} {output}");
  println!("\nSend '~' to exit\n");

  const FREQ: u32 = 50;
  const MAX_V: f32 = 3.3;

  // Validating pwm pin
  let (pwm_id, channel) = device.pwms.get_pwm_slice_id_by_gpio(gpio_output)?;

  // Initializing PWM slice
  with_pwm_slice!(&mut device.pwms, pwm_id, |pwm_slice| {
    pwm_slice.set_freq(FREQ);
    pwm_slice.enable();
  });

  let pwm_pin = &mut device.pwms.get_channel_by_gpio(gpio_output).unwrap();

  // Loop
  SERIAL.clear_interrupt_cmd();
  while !SERIAL.interrupt_cmd_triggered() {
    if let Some(raw) = device.adcs.read_by_gpio_id(gpio_input) {
      // Analog Read - Clamping 0.3V deadzone from both ends
      let factor = (raw.to_voltage() - 0.3).clamp(0.0, MAX_V - 0.6) / (MAX_V - 0.6);

      // Defined us range
      if min_us > 0 && max_us > 0 {
        let us = min_us + ((max_us - min_us) as f32 * factor) as u16;
        pwm_pin.set_duty_cycle_us(us, FREQ);
      }
      // Fraction range
      else if factor == 0.0 {
        let _ = pwm_pin.set_duty_cycle_fully_off();
      }
      else if factor == 1.0 {
        let _ = pwm_pin.set_duty_cycle_fully_on();
      }
      else {
        let _ = pwm_pin.set_duty_cycle_fraction((factor * u16::MAX as f32) as u16, u16::MAX);
      }
    }
  }

  pwm_pin.set_duty_cycle_fully_off().unwrap();
  println!("Done!");
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Panic Test
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_test_panic_cmd() -> Command {
  Command {
    name: "test_panic",
    desc: "Panics the program",
    help: "test_panic [help]",
    func: test_panic_cmd,
  }
}

pub fn test_panic_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  print!("\n On the next boot you should see the msg \"PANIC TEST\"\n Panicking.... :O\n");
  panic!("PANIC TEST");
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Test Log
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_test_log_cmd() -> Command {
  Command {
    name: "test_log",
    desc: "Test the logging system",
    help: "test_log [help] ",
    func: test_log_cmd,
  }
}

pub fn test_log_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  println!("Log Level: {}\n", LOG.get());

  error!("Error");
  warn!("Warn");
  info!("Info");
  debug!("Debug");
  trace!("Trace");

  Ok(())
}
