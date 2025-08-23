//! CLI Commands/Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          CLI Commands
// ————————————————————————————————————————————————————————————————————————————————————————————————

use super::*;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Commands List
// ————————————————————————————————————————————————————————————————————————————————————————————————

const NUM_COMMANDS: usize = 9;

pub const CMDS: [Command; NUM_COMMANDS] = [
  Command {
    name: "help",
    desc: "Show command help",
    func: help_cmd,
  },
  Command {
    name: "reset",
    desc: "Reset device",
    func: reset_cmd,
  },
  Command {
    name: "flash",
    desc: "Restart device in USB Flash mode",
    func: flash_cmd,
  },
  Command {
    name: "example",
    desc: "Print Example | <arg(float)> [opt=0(u8)] [on=false(bool)] [path=\"\"(string)]",
    func: example_cmd,
  },
  Command {
    name: "blink",
    desc: "Blink Onboard Led | [times=10]",
    func: blink_cmd,
  },
  Command {
    name: "read_adc",
    desc: "Read ADC | [ref_res=10000(ohm)]",
    func: read_adc_cmd,
  },
  Command {
    name: "sample_adc",
    desc: "Samples ADC GPIO 26 | [channel=0(u8)] [ref_res=10000(ohm)]",
    func: sample_adc_cmd,
  },
  Command {
    name: "servo",
    desc: "Set Servo PWM on GPIO 2 | [us=1500(us)] [pause=2000(ms)]",
    func: servo_cmd,
  },
  Command {
    name: "set_pwm",
    desc: "Sets PWM on GPIO 6 | [freq=50(hz)] [us=-1(us)] [duty=50(%)] \n [top=-1(u16)] \
           [phase=false(bool)][disable=false(bool)]",
    func: set_pwm_cmd,
  },
];

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Commands Config
// ————————————————————————————————————————————————————————————————————————————————————————————————

type Result<T> = core::result::Result<T, CliError>;
type Context = Device;

// ———————————————————————————————————————————— Help ——————————————————————————————————————————————

fn help_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  println!("Available commands:\n");
  for cmd in CMDS.iter() {
    println!(" {} - {} ", &cmd.name, &cmd.desc);
  }
  Ok(())
}

// ———————————————————————————————————————————— Test ——————————————————————————————————————————————

fn example_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
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

// ——————————————————————————————————————————— Reset ——————————————————————————————————————————————

fn reset_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  print!("\nResetting...\n");
  DELAY.ms(1500); // Waiting for reset msg to appear
  device_reset();
  Ok(())
}

// ——————————————————————————————————————————— Flash ——————————————————————————————————————————————

fn flash_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  print!("\nRestarting in USB Flash mode!...\n");
  device_reset_to_usb();

  Ok(())
}

// —————————————————————————————————————————— Blink —————————————————————————————————————————————
// ex: blink times=4

fn blink_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  let times: u8 = get_parsed_param("times", args).unwrap_or(10); // 10 default
  blink(device, times)
}

// Separating functions from commands for stand alone use
fn blink(device: &mut Context, times: u8) -> Result<()> {
  println!("---- Blinking Led! ----");

  for n in 1..(times + 1) {
    print!("Blink {} | ", n);
    device.outputs.led.set_high().unwrap();
    device.timer.delay_ms(200);
    device.outputs.led.set_low().unwrap();
    device.timer.delay_ms(200);
  }

  Ok(())
}

// —————————————————————————————————————————— Read ADC —————————————————————————————————————————————

fn read_adc_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  let ref_res: u32 = get_parsed_param("ref_res", args).unwrap_or(10_000);

  read_adc(device, ref_res)
}

fn read_adc(device: &mut Context, ref_res: u32) -> Result<()> {
  println!("---- Read ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);

  let channels_to_read: [u8; _] = [0, 1, 2, 3];

  for &channel in &channels_to_read {
    if let Some(r) = device.acds.read_channel(channel) {
      let adc_raw = r;
      let adc_vol = adc_raw.to_voltage();
      let adc_res = adc_raw.to_resistance(ref_res);
      println!("ACD {}: {}, {:.2}V, {:.1}ohm ", channel, adc_raw, adc_vol, adc_res);
    }
  }

  // read Temp Sense
  let adc_raw: u16 = device.acds.read_channel(TEMP_SENSE_CHN).unwrap_or(0);
  let adc_vol = adc_raw.to_voltage();
  let adc_res = adc_raw.to_resistance(ref_res);
  let sys_temp = 27.0 - (adc_raw.to_voltage() - 0.706) / 0.001721;
  println!("Temp Sense: {}, {:.2}V, {:.1}C", adc_raw, adc_vol, sys_temp);

  Ok(())
}

// ————————————————————————————————————————— Sample Adc ———————————————————————————————————————————
//GPIO 26

fn sample_adc_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  let ref_res: u32 = get_parsed_param("ref_res", args).unwrap_or(10_000);
  let channel: u8 = get_parsed_param("channel", args).unwrap_or(0);

  sample_adc(device, channel, ref_res)
}

fn sample_adc(device: &mut Context, channel: u8, ref_res: u32) -> Result<()> {
  println!("---- Sample ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);
  println!("ADC Channel: {} \n", { channel });

  while !SERIAL.poll_for_break_cmd() {
    if let Some(r) = device.acds.read_channel(channel) {
      let adc_raw: u16 = r;
      let adc_vol = adc_raw.to_voltage();
      let adc_res = adc_raw.to_resistance(ref_res);
      println!("> v:{:.2}, ohm:{:.1}, raw:{} \r", adc_vol, adc_res, adc_raw);
      device.timer.delay_ms(400);
    } else {
      println!("Cannot read channel: {}", channel);
    }
  }

  println!("Sampling Interrupted. Done!");

  Ok(())
}

// —————————————————————————————————————————— Servo —————————————————————————————————————————————
// GPIO 2 A
// ex: servo us=1200 pause=1000

fn servo_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  let us: u16 = get_parsed_param("us", args).unwrap_or(1500); //  1500 us default
  let pause: u16 = get_parsed_param("pause", args).unwrap_or(2000); // 2s default

  servo(device, us, pause)
}

fn servo(device: &mut Context, us: u16, pause: u16) -> Result<()> {
  const CYCLE: u16 = 20 * 1000; // 20ms - 50hz

  println!("---- Servo ----");

  let us = if CYCLE <= us { CYCLE } else { us };
  let pwm_pin = &mut device.pwms.pwm_1.channel_a; // GPIO 2 A

  println!("Setting PWM to: {}us, {}%  ... ", us, ((us as f32 / CYCLE as f32) * 100.0));
  pwm_pin.enable();
  pwm_pin.set_duty_cycle_fraction(us, CYCLE).unwrap();
  device.timer.delay_ms(pause as u32);
  pwm_pin.disable();
  println!("Done!");

  Ok(())
}

// —————————————————————————————————————————— Set PWM —————————————————————————————————————————————
// GPIO 6 A
// ex:

fn set_pwm_cmd(args: &[Arg], device: &mut Context) -> Result<()> {
  let us: i32 = get_parsed_param("us", args).unwrap_or(-1); //  -1 eq not set
  let duty: u8 = get_parsed_param("duty", args).unwrap_or(50); //  50% default
  let freq: u32 = get_parsed_param("freq", args).unwrap_or(50); // hz
  let top: i32 = get_parsed_param("top", args).unwrap_or(-1); // 
  let phase: bool = get_parsed_param("phase", args).unwrap_or(false); // 
  let disable: bool = get_parsed_param("disable", args).unwrap_or(false); // false

  set_pwm(device, us, duty, freq, top, phase, disable)
}

fn set_pwm(
  device: &mut Context, us: i32, duty: u8, freq: u32, top: i32, phase: bool, disable: bool,
) -> Result<()> {
  println!("---- PWM ----");

  let pwm = &mut device.pwms.pwm_3; // GPIO 

  if disable {
    pwm.disable();
    println!("PWM Pin disabled");
    return Ok(());
  }

  // Setting PWM
  pwm.disable();

  if phase {
    pwm.set_ph_correct()
  } else {
    pwm.clr_ph_correct();
  }

  let top = if top < 0 { pwm.get_top() } else { top.clamp(0, u16::MAX as i32) as u16 };
  pwm.set_top(top);

  let (int, frac) = calculate_pwm_dividers(freq as f32, top, phase);
  pwm.set_div_int(int);
  pwm.set_div_frac(frac);

  print!("Seting PWM | freq: {}hz, top: {}, phase: {}", freq, top, phase);

  // Duty cycle
  // setting duty percentage if us not defined (-1)
  if us < 0 {
    pwm.channel_a.set_duty_cycle_percent(duty).unwrap();
    println!(", duty {}%", duty.clamp(0, 100));
  } else {
    // setting duty cycle to us size
    let freq_us = (1_000_000 / freq) as u16;
    let us = us.clamp(0, freq_us as i32) as u16;
    pwm.channel_a.set_duty_cycle_fraction(us, freq_us).unwrap();
    println!(", duty {}µs", us);
  }

  // End
  pwm.enable();
  Ok(())
}
