//! Example Commands
// Register new commands in commands.rs > Command List Builder

use super::*;

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

pub fn example_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
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
// ex: blink times=4

pub fn build_blink_cmd() -> Command {
  Command {
    name: "blink",
    desc: "Blinks Onboard Led",
    help: "blink [times=10] [interval=200(ms)] [help]",
    func: blink_cmd,
  }
}

pub fn blink_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let times: u16 = args.get_parsed_param("times").unwrap_or(10); // 10 default
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200); // 200ms default
  blink(device, times, interval)
}

// Separating functions from commands for stand alone use
pub fn blink(device: &mut Device, times: u16, interval: u16) -> Result<()> {
  println!("---- Blinking Led! ----");
  let led = device.outputs.get_by_id(PinID::LED).unwrap();
  let mut blink = 1;

  // Non blocking timer based task
  let mut ledtask = Tasklet::new(interval as u32, times * 2, &device.timer);

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

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Panic Test
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_panic_test_cmd() -> Command {
  Command {
    name: "panic_test",
    desc: "Panics the program",
    help: "panic_test [help]",
    func: panic_test_cmd,
  }
}

pub fn panic_test_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  print!("\n On the next boot you should see the msg \"PANIC TEST\"\n Panicking.... :O\n");
  panic!("PANIC TEST");
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Servo
// —————————————————————————————————————————————————————————————————————————————————————————————————
// GPIO 8 pwm4A
// Angle Controlled RC Servo
// ex: servo us=1200 pause=1000
// ex: servo sweep=true max_us=1800

pub fn build_servo_cmd() -> Command {
  Command {
    name: "servo",
    desc: "Set Servo PWM on GPIO 8",
    help: "servo [us=1500(us)] [pause=1500(ms)] [sweep=false(bool)] [max_us=2000(us)] [help]",
    func: servo_cmd,
  }
}

pub fn servo_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let us: u16 = args.get_parsed_param("us").unwrap_or(1500); //  1500 us default
  let pause: u32 = args.get_parsed_param("pause").unwrap_or(1500); // 2s default
  let sweep: bool = args.get_parsed_param("sweep").unwrap_or(false); //  false default
  let max_us: u16 = args.get_parsed_param("max_us").unwrap_or(2000); //  2000 us default

  servo(device, us, pause, sweep, max_us)
}

pub fn servo(device: &mut Device, us: u16, pause: u32, sweep: bool, max_us: u16) -> Result<()> {
  println!("---- Servo ----");
  println!("GPIO 8 pwm4A");

  const FREQ: u32 = 50;
  const MID: u16 = 1500; // Home position

  let servo_pwm = &mut device.pwms.pwm4; // GPIO 8 pwm4A

  let max_us = max_us.clamp(MID, max_us);
  let min_us = MID - (max_us - MID).clamp(1, MID);

  println!("Setting PWM: Duty: {}us, Freq: {}", us, FREQ);
  servo_pwm.set_freq(FREQ);

  let servo_pin = servo_pwm.get_channel_a();
  servo_pin.set_duty_cycle_us(us, FREQ);
  servo_pwm.enable();
  device.timer.delay_ms(pause);

  let servo_pin = servo_pwm.get_channel_a();

  if sweep {
    // resetting borrow
    println!("Sweeping...");
    // Max
    servo_pin.set_duty_cycle_us(max_us, FREQ);
    println!("Max: {max_us}us");
    device.timer.delay_ms(pause);

    // Mid
    servo_pin.set_duty_cycle_us(MID, FREQ);
    println!("Mid: {MID}us");
    device.timer.delay_ms(pause);

    // Min
    servo_pin.set_duty_cycle_us(min_us, FREQ);
    println!("Min: {min_us}us");
    device.timer.delay_ms(pause);

    // Mid
    servo_pin.set_duty_cycle_us(MID, FREQ);
    println!("Mid: {MID}us");
    device.timer.delay_ms(pause);
  }

  // Off
  servo_pin.set_duty_cycle_fully_off().unwrap();
  servo_pwm.disable();
  println!("Done!");
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Test GPIO
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_test_gpio_cmd() -> Command {
  Command {
    name: "test_gpio",
    desc: "Sets GPIO 0 High when GPIO 9 is Low",
    help: "test_gpio [help] \nInterrupt with char \"~\" ",
    func: test_gpio_cmd,
  }
}

pub fn test_gpio_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }
  println!("---- Testing GPIO ----");
  println!("Send '~' to exit");

  let input = device.inputs.get_by_id(PinID::IN_A).unwrap();
  let output = device.outputs.get_by_id(PinID::OUT_A).unwrap();

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

pub fn build_test_analog_cmd() -> Command {
  Command {
    name: "test_analog",
    desc: "Voltage controlled PWM Duty Cycle",
    help: "test_analog [help] \nInterrupt with char \"~\" ",
    func: test_analog_cmd,
  }
}

pub fn test_analog_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }
  println!("---- Testing Analog Input ----");
  println!("Input: GPIO 26 >> PWM Output: GPIO 8");
  println!("Send '~' to exit");

  const FREQ: u32 = 60;
  const MAX_V: f32 = 3.3;

  let adc_channel = 0; // GPIO 26 - adc0
  let pwm = &mut device.pwms.pwm4; // GPIO 8 - pwm4A

  // PWM setup
  pwm.set_freq(FREQ);
  pwm.get_channel_a().set_duty_cycle_fully_off().unwrap();
  pwm.enable();

  let pwm_channel = pwm.get_channel_a();

  SERIAL.clear_interrupt_cmd();
  while !SERIAL.interrupt_cmd_triggered() {
    // Analog Read
    if let Some(r) = device.adcs.read_channel(adc_channel) {
      let adc_v = r.to_voltage().clamp(0.0, MAX_V);

      // PWM
      let _ = match adc_v {
        MAX_V => pwm_channel.set_duty_cycle_fully_on(),
        v if v < 0.1 => pwm_channel.set_duty_cycle_fully_off(),
        _ => pwm_channel.set_duty_cycle_fraction((adc_v * 100.0) as u16, (MAX_V * 100.0) as u16),
      };
    }
  }

  // Off
  pwm_channel.set_duty_cycle_fully_off().unwrap();
  pwm.disable();
  println!("Done!");
  Ok(())
}
