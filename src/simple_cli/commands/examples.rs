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
    func: example_cmd,
  }
}

pub fn example_cmd_help() {
  println!("Help: example");
  println!(
    "Prints Example Arguments\n
    example <arg(float)> [opt=0(u8)] [on=false(bool)] [path=\"\"(string)] [help]"
  )
}

pub fn example_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    example_cmd_help();
    return Ok(());
  }

  let arg: f32 = get_parsed_param("arg", args)?; // required argument
  let opt: u8 = get_parsed_param("opt", args).unwrap_or(0); // argument with default
  let on: bool = get_parsed_param("on", args).unwrap_or(false);
  let path: &str = get_str_param("path", args).unwrap_or("");

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
    func: blink_cmd,
  }
}

pub fn blink_cmd_help() {
  println!("Help: blink");
  println!(
    "Blinks Onboard Led \n
    blink [times=10] [interval=200(ms)] [help]"
  )
}

pub fn blink_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    blink_cmd_help();
    return Ok(());
  }

  let times: u16 = get_parsed_param("times", args).unwrap_or(10); // 10 default
  let interval: u16 = get_parsed_param("interval", args).unwrap_or(200); // 10 default
  blink(device, times, interval)
}

// Separating functions from commands for stand alone use
pub fn blink(device: &mut Device, times: u16, interval: u16) -> Result<()> {
  println!("---- Blinking Led! ----");
  let led = device.outputs.get_pin(PinID::LED).unwrap();
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
  // for n in 1..(times + 1) {
  //   print!("Blink {} | ", n);
  //   led.set_high().unwrap();
  //   device.timer.delay_ms(200);
  //   led.set_low().unwrap();
  //   device.timer.delay_ms(200);
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
    func: panic_test_cmd,
  }
}

pub fn panic_test_cmd_help() {
  println!("Help: panic_test");
  println!(
    "Panics the program. On the next serial connection, the panic msg is printed\n
    panic_test [help]"
  )
}

pub fn panic_test_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    panic_test_cmd_help();
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
    func: servo_cmd,
  }
}

pub fn servo_cmd_help() {
  println!("Help: servo");
  println!(
    "Set Servo PWM on GPIO 8\n
    servo [us=1500(us)] [pause=1500(ms)] [sweep=false(bool)] [max_us=2000(us)] [help]"
  )
}

pub fn servo_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    servo_cmd_help();
    return Ok(());
  }

  let us: u16 = get_parsed_param("us", args).unwrap_or(1500); //  1500 us default
  let pause: u32 = get_parsed_param("pause", args).unwrap_or(1500); // 2s default
  let sweep: bool = get_parsed_param("sweep", args).unwrap_or(false); //  false default
  let max_us: u16 = get_parsed_param("max_us", args).unwrap_or(2000); //  2000 us default

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
    func: test_gpio_cmd,
  }
}

pub fn test_gpio_cmd_help() {
  println!("Help: test_gpio");
  println!(
    "Sets GPIO 0 High when GPIO 9 is Low\n
   test_gpio [help] \n
   Interrupt with char \"~\" "
  )
}

pub fn test_gpio_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    test_gpio_cmd_help();
    return Ok(());
  }

  println!("---- Testing GPIO ----");
  println!("Send '~' to exit");

  let input = device.inputs.get_pin(PinID::IN_A).unwrap();
  let output = device.outputs.get_pin(PinID::OUT_A).unwrap();

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
    func: test_analog_cmd,
  }
}

pub fn test_analog_cmd_help() {
  println!("Help: test_analog");
  println!(
    "Voltage controlled PWM Duty Cycle (i.e. Potentiometer on GPIO 26 dimming a Led on GPIO 8)\n
    test_analog [help] \n
    Interrupt with char \"~\" "
  )
}

pub fn test_analog_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    test_analog_cmd_help();
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
