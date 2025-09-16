use super::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                      Command List Builder
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_commands() -> CommandList {
  let mut list = CommandList::default();

  list.register_command(Command {
    name: "help",
    desc: "Show command help - [command_name=all]",
    func: help_cmd,
  });

  list.register_command(Command {
    name: "reset",
    desc: "Reset device - [help]",
    func: reset_cmd,
  });

  list.register_command(Command {
    name: "flash",
    desc: "Restart device in USB Flash mode - [help]",
    func: flash_cmd,
  });

  list.register_command(Command {
    name: "example",
    desc: "Print Example \n <arg(float)> [opt=0(u8)] [on=false(bool)] [path=\"\"(string)] [help]",
    func: example_cmd,
  });

  list.register_command(Command {
    name: "blink",
    desc: "Blink Onboard Led \n [times=10] [interval=200(ms)] [help]",
    func: blink_cmd,
  });

  list.register_command(Command {
    name: "read_adc",
    desc: "Read all ADC channels \n [ref_res=10000(ohm)] [help]",
    func: read_adc_cmd,
  });

  list.register_command(Command {
    name: "sample_adc",
    desc: "Continuous sampling of an ADC channel \n [channel=0(u8)] [ref_res=10000(ohm)] \
           [interval=200(ms)] [help]\n Interrupt with char \"~\" ",
    func: sample_adc_cmd,
  });

  list.register_command(Command {
    name: "servo",
    desc: "Set Servo PWM on GPIO 8 \n [us=1500(us)] [pause=1500(ms)] [sweep=false(bool)] \
           [max_us=2000(us)] [help]",
    func: servo_cmd,
  });

  list.register_command(Command {
    name: "set_pwm",
    desc: "Sets PWM  (defaults on GPIO 6 - PWM3A ) \n [pwm_id=3(id)] [channel=a(a/b)] \
           [freq=50(hz)] [us=-1(us)] [duty=50(%)] \n [top=-1(u16)] [phase=false(bool)] \
           [disable=false(bool)] [help]",
    func: set_pwm_cmd,
  });

  list.register_command(Command {
    name: "panic_test",
    desc: "Panics the program. On the next serial connection, the panic msg is printed - [help]",
    func: panic_test_cmd,
  });

  list.register_command(Command {
    name: "test_gpio",
    desc: "Sets GPIO 0 High when GPIO 9 is Low - [help] \n Interrupt with char \"~\" ",
    func: test_gpio_cmd,
  });

  list.register_command(Command {
    name: "test_analog",
    desc: "Voltage controlled PWM Duty Cycle (i.e. Potentiometer on GPIO 26 dimming a Led on GPIO \
           8) - [help] \n Interrupt with char \"~\" ",
    func: test_analog_cmd,
  });

  list
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Commands
// —————————————————————————————————————————————————————————————————————————————————————————————————

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Help
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn help_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  let mut input = "all";
  if let Some(arg) = &args.first() {
    input = &arg.param;
  }
  else {
    return help(input, commands);
  }

  if let Ok(cmd_desc) = commands.get_description(input) {
    help(input, commands)
  }
  else {
    Err(CliError::CmdNotFound(String::from_str(input).unwrap()))
  }
}

pub fn help(command: &str, commands: &CommandList) -> Result<()> {
  if command == "all" {
    println!("All available commands:\n");
    for cmd in commands.commands.iter() {
      println!(" {} - {} \n", &cmd.name, &cmd.desc);
    }
    return Ok(());
  }

  if !command.is_empty()
    && let Ok(desc) = commands.get_description(command)
  {
    println!(" Help: {} \n", command);
    println!(" {} - {} \n", command, desc);
    Ok(())
  }
  else {
    Err(CliError::CmdNotFound(String::from_str(command).unwrap()))
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Example
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn example_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("example", commands);
  }

  let arg: f32 = get_parsed_param("arg", args)?;
  let opt: u8 = get_parsed_param("opt", args).unwrap_or(0); // With default
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
//                                              Reset
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn reset_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("reset", commands);
  }

  print!("\nResetting...\n");
  device.timer.delay_ms(500); // Waiting for msg to appear
  device_reset();
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Flash
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn flash_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("flash", commands);
  }

  print!("\nRestarting in USB Flash mode!...\n");
  device_reset_to_usb();

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Blink
// —————————————————————————————————————————————————————————————————————————————————————————————————
// ex: blink times=4

pub fn blink_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("blink", commands);
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
//                                            Read ADC
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn read_adc_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("read_adc", commands);
  }

  let ref_res: u32 = get_parsed_param("ref_res", args).unwrap_or(10_000);

  read_adc(device, ref_res)
}

pub fn read_adc(device: &mut Device, ref_res: u32) -> Result<()> {
  println!("---- Read ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);

  let channels_to_read: [u8; _] = [0, 1, 2, 3];

  for &channel in &channels_to_read {
    if let Some(r) = device.adcs.read_channel(channel) {
      let adc_raw = r;
      let adc_vol = adc_raw.to_voltage();
      let adc_res = adc_raw.to_resistance(ref_res);
      println!("> ACD {}: v:{:.2}, ohm:{:.1}, raw:{} \r", channel, adc_vol, adc_res, adc_raw);
    }
  }

  // read Temp Sense
  let adc_raw: u16 = device.adcs.read_channel(TEMP_SENSE_CHN).unwrap_or(0);
  let adc_vol = adc_raw.to_voltage();
  let adc_res = adc_raw.to_resistance(ref_res);
  let sys_temp = 27.0 - (adc_raw.to_voltage() - 0.706) / 0.001721;
  println!("Temp Sense: C:{:.1}, v:{:.2}, raw:{}", sys_temp, adc_vol, adc_raw);

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Sample ADC
// —————————————————————————————————————————————————————————————————————————————————————————————————
// GPIO 26

pub fn sample_adc_cmd(
  args: &[Arguments],
  device: &mut Device,
  commands: &CommandList,
) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("sample_adc", commands);
  }

  let ref_res: u32 = get_parsed_param("ref_res", args).unwrap_or(10_000);
  let channel: u8 = get_parsed_param("channel", args).unwrap_or(0);
  let interval: u16 = get_parsed_param("interval", args).unwrap_or(200);

  sample_adc(device, channel, ref_res, interval)
}

pub fn sample_adc(device: &mut Device, channel: u8, ref_res: u32, interval: u16) -> Result<()> {
  println!("---- Sample ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);
  println!("ADC Channel: {} \n", { channel });

  SERIAL.clear_interrupt_cmd();
  while !SERIAL.interrupt_cmd_triggered() {
    if let Some(r) = device.adcs.read_channel(channel) {
      let adc_raw: u16 = r;
      let adc_vol = adc_raw.to_voltage();
      let adc_res = adc_raw.to_resistance(ref_res);
      println!("> v:{:.2}, ohm:{:.1}, raw:{} \r", adc_vol, adc_res, adc_raw);
      device.timer.delay_ms(interval as u32);
    }
    else {
      println!("Cannot read channel: {}", channel);
    }
  }

  println!("Sampling Interrupted. Done!");

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Servo
// —————————————————————————————————————————————————————————————————————————————————————————————————

// GPIO 8 pwm4A
// Angle Controlled RC Servo
// ex: servo us=1200 pause=1000
// ex: servo sweep=true max_us=1800

pub fn servo_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("servo", commands);
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
//                                             Set PWM
// —————————————————————————————————————————————————————————————————————————————————————————————————
// GPIO 6 pwm3A

pub fn set_pwm_cmd(args: &[Arguments], device: &mut Device, commands: &CommandList) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("set_pwm", commands);
  }

  let pwm_id: usize = get_parsed_param("pwm_id", args).unwrap_or(3); //  -1 eq not set
  let channel = get_str_param("channel", args).unwrap_or("a"); // false
  let us: i32 = get_parsed_param("us", args).unwrap_or(-1); //  -1 eq not set
  let duty: u8 = get_parsed_param("duty", args).unwrap_or(50); //  50% default
  let freq: u32 = get_parsed_param("freq", args).unwrap_or(50); // hz
  let top: i32 = get_parsed_param("top", args).unwrap_or(-1); // 
  let phase: bool = get_parsed_param("phase", args).unwrap_or(false); // 
  let disable: bool = get_parsed_param("disable", args).unwrap_or(false); // false

  if channel != "a" && channel != "b" {
    println!("Channel can be only a or b");
    return Err(CliError::Exit);
  }

  println!("---- PWM ----");
  println!("PWM: {pwm_id}, channel: {channel}");

  // Using a 'with' macro to be able to select the PWM slice
  // In a regular program you would use the pwm slice directly
  with_pwm_slice!(&mut device.pwms, pwm_id, |pwm_slice| {
    set_pwm(pwm_slice, channel, us, duty, freq, top, phase, disable)
  })
}

use embedded_hal::digital::InputPin;
use rp_pico::hal::pwm;

#[allow(clippy::too_many_arguments)]
pub fn set_pwm<I>(
  pwm_slice: &mut crate::pwms::PwmSlice<I>,
  channel: &str,
  us: i32,
  duty: u8,
  freq: u32,
  top: i32,
  phase: bool,
  disable: bool,
) -> Result<()>
where
  I: pwm::SliceId,
  <I as pwm::SliceId>::Reset: pwm::ValidSliceMode<I>,
{
  //
  if disable {
    pwm_slice.disable();
    println!("PWM Pin disabled");
    return Ok(());
  }

  // Set PWM
  if pwm_slice.ph_correct != phase {
    pwm_slice.set_ph_correct(phase);
  }

  // Set TOP
  let top = if top > 0 { top.clamp(0, u16::MAX as i32) as u16 } else { u16::MAX };
  if pwm_slice.slice.get_top() != top {
    pwm_slice.set_top(top);
  }

  // Set Frequency
  if pwm_slice.freq != freq {
    pwm_slice.set_freq(freq);
  }

  print!("Seting PWM | freq: {}hz, top: {}, phase: {} ", freq, top, phase);

  // Set Duty
  if us > 0 {
    if channel == "a" {
      pwm_slice.get_channel_a().set_duty_cycle_us(us as u16, freq);
    }
    else {
      pwm_slice.get_channel_b().set_duty_cycle_us(us as u16, freq);
    }

    println!("duty: {}µs", us);
  }
  else {
    let duty = duty.clamp(0, 100) as u16;
    if channel == "a" {
      pwm_slice.get_channel_a().set_duty_cycle_fraction(duty, 100).unwrap();
    }
    else {
      pwm_slice.get_channel_b().set_duty_cycle_fraction(duty, 100).unwrap();
    }

    println!("duty: {}%", duty);
  }

  // End
  pwm_slice.enable();

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Panic Test
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn panic_test_cmd(
  args: &[Arguments],
  device: &mut Device,
  commands: &CommandList,
) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("panic_test", commands);
  }

  print!("\n On the next boot you should see the msg \"PANIC TEST\"\n Panicking.... :O\n");
  panic!("PANIC TEST");
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Test GPIO
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn test_gpio_cmd(
  args: &[Arguments],
  device: &mut Device,
  commands: &CommandList,
) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("test_gpio", commands);
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

pub fn test_analog_cmd(
  args: &[Arguments],
  device: &mut Device,
  commands: &CommandList,
) -> Result<()> {
  // Print Help
  if let Some(arg) = args.iter().find(|arg| arg.param.contains("help")) {
    return help("test_analog", commands);
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
