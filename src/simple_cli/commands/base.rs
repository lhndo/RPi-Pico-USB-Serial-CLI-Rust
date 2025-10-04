//! Core commands
// Register new commands in commands.rs > Command List Builder

use super::*;
use crate::prelude::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Reset
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_reset_cmd() -> Command {
  Command {
    name: "reset",
    desc: "Resets Device",
    help: "reset [help]",
    func: reset_cmd,
  }
}

pub fn reset_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  print!("\nResetting...\n");
  device.timer.delay_ms(500); // Waiting for msg to appear
  device_reset();
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Flash
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_flash_cmd() -> Command {
  Command {
    name: "flash",
    desc: "Restart device in USB Flash mode",
    help: "flash [help]",
    func: flash_cmd,
  }
}

pub fn flash_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  print!("\nRestarting in USB Flash mode!...\n");
  device_reset_to_usb();

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Set Pin
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_set_pin_cmd() -> Command {
  Command {
    name: "set_pin",
    desc: "Set GPIO Pin State",
    help: "set_pin [gpio=1(u8)] [toggle] [high] [low] [help]",
    func: set_pin_cmd,
  }
}

pub fn set_pin_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let gpio_id: u8 = args.get_parsed_param("gpio").unwrap_or(1);
  let high = args.contains_param("high");
  let low = args.contains_param("low");

  if let Some(pin) = device.outputs.get_by_gpio_id(gpio_id) {
    if high {
      println!("GPIO {gpio_id}: Set HIGH");
      pin.set_high().unwrap();
    }
    else if low {
      println!("GPIO {gpio_id}: Set LOW");
      pin.set_low().unwrap();
    }
    else {
      print!("GPIO {gpio_id}: Toggled ");
      pin.toggle().unwrap();
      if pin.is_set_high().unwrap() {
        println!("HIGH")
      }
      else {
        println!("LOW")
      }
    }
  }
  else {
    return Err(CliError::CmdExec("GPIO pin not configured".into_truncated()));
  }
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Read Pin
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_read_pin_cmd() -> Command {
  Command {
    name: "read_pin",
    desc: "Set GPIO Pin State",
    help: "read_pin [gpio=1(u8)]",
    func: read_pin_cmd,
  }
}

pub fn read_pin_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let gpio_id: u8 = args.get_parsed_param("gpio").unwrap_or(1);

  if let Some(pin) = device.outputs.get_by_gpio_id(gpio_id) {
    print!("GPIO {gpio_id}: ");
    if pin.is_set_high().unwrap() {
      println!("HIGH");
    }
    else {
      println!("LOW");
    }
  }
  else if let Some(pin) = device.outputs.get_by_gpio_id(gpio_id) {
    print!("GPIO {gpio_id}: ");
    if pin.is_set_high().unwrap() {
      println!("HIGH");
    }
    else {
      println!("LOW");
    }
  }
  else {
    return Err(CliError::CmdExec("GPIO pin not configured".into_truncated()));
  }
  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Read ADC
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_read_adc_cmd() -> Command {
  Command {
    name: "read_adc",
    desc: "Read all ADC channels",
    help: "read_adc [ref_res=10000(ohm)] [help]",
    func: read_adc_cmd,
  }
}

pub fn read_adc_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let ref_res: u32 = args.get_parsed_param("ref_res").unwrap_or(10_000);
  read_adc(device, ref_res)
}

pub fn read_adc(device: &mut Device, ref_res: u32) -> Result<()> {
  println!("---- Read ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);

  let channels_to_read: [u8; _] = [0, 1, 2, 3];

  for &channel in &channels_to_read {
    if let Some(r) = device.adcs.read(channel) {
      let adc_raw = r;
      let adc_vol = adc_raw.to_voltage();
      let adc_res = adc_raw.to_resistance(ref_res);
      println!("> ACD {}: v:{:.2}, ohm:{:.1}, raw:{} \r", channel, adc_vol, adc_res, adc_raw);
    }
  }

  // read Temp Sense
  let adc_raw: u16 = device.adcs.read(TEMP_SENSE_CHN).unwrap_or(0);
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

pub fn build_sample_adc_cmd() -> Command {
  Command {
    name: "sample_adc",
    desc: "Continuous sampling of an ADC channel",
    help: "sample_adc [gpio(u8)] or [channel=0(u8)]  [ref_res=10000(ohm)] [interval=200(ms)] \
           [help]\n
    Interrupt with char \"~\"",
    func: sample_adc_cmd,
  }
}

pub fn sample_adc_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let ref_res: u32 = args.get_parsed_param("ref_res").unwrap_or(10_000);
  let mut channel: u8 = args.get_parsed_param("channel").unwrap_or(0);
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200);

  // Getting ADC channel based on pin number
  if let Ok(gpio_) = args.get_parsed_param("gpio") {
    match gpio_ {
      26 => channel = 0,
      27 => channel = 1,
      28 => channel = 2,
      29 => channel = 3,
      255 => channel = 4, // default TEMP_SENSE channel
      _ => return Err(CliError::CmdExec("pin not configured for ADC read".into_truncated())),
    }
  }

  sample_adc(device, channel, ref_res, interval)
}

pub fn sample_adc(device: &mut Device, channel: u8, ref_res: u32, interval: u16) -> Result<()> {
  println!("---- Sample ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);
  println!("ADC Channel: {} \n", { channel });
  println!("Send '~' to exit\n");

  SERIAL.clear_interrupt_cmd();
  while !SERIAL.interrupt_cmd_triggered() {
    if let Some(r) = device.adcs.read(channel) {
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
//                                             Set PWM
// —————————————————————————————————————————————————————————————————————————————————————————————————
// GPIO 6 pwm3A

pub fn build_set_pwm_cmd() -> Command {
  Command {
    name: "set_pwm",
    desc: "Sets PWM  (defaults on GPIO 6 - PWM3A)",
    help:
      "set_pwm [pwm_id=3(id)] [channel=a(a/b)] [freq=50(hz)] [us=-1(us)] [duty=50(%)]\n        \
       [top=-1(u16)] [phase=false(bool)] [disable=false(bool)] [help]",
    func: set_pwm_cmd,
  }
}

pub fn set_pwm_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let pwm_id: usize = args.get_parsed_param("pwm_id").unwrap_or(3); //  -1 eq not set
  let channel = args.get_str_param("channel").unwrap_or("a"); // false
  let us: i32 = args.get_parsed_param("us").unwrap_or(-1); //  -1 eq not set
  let duty: u8 = args.get_parsed_param("duty").unwrap_or(50); //  50% default
  let freq: u32 = args.get_parsed_param("freq").unwrap_or(50); // hz
  let top: i32 = args.get_parsed_param("top").unwrap_or(-1); // 
  let phase: bool = args.get_parsed_param("phase").unwrap_or(false); // 
  let disable: bool = args.get_parsed_param("disable").unwrap_or(false); // false

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

use rp2040_hal::pwm;

#[allow(clippy::too_many_arguments)]
pub fn set_pwm<I>(
  pwm: &mut crate::pwms::PwmSlice<I>,
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
    pwm.disable();
    println!("PWM Pin disabled");
    return Ok(());
  }

  // Set PWM
  if pwm.ph_correct != phase {
    pwm.set_ph_correct(phase);
  }

  // Set TOP
  let top = if top > 0 { top.clamp(0, u16::MAX as i32) as u16 } else { u16::MAX };
  if pwm.slice.get_top() != top {
    pwm.set_top(top);
  }

  // Set Frequency
  if pwm.freq != freq {
    pwm.set_freq(freq);
  }

  print!("Seting PWM | freq: {}hz, top: {}, phase: {} ", freq, top, phase);

  // Set Duty
  if us > 0 {
    if channel == "a" {
      pwm.get_channel_a().set_duty_cycle_us(us as u16, freq);
    }
    else {
      pwm.get_channel_b().set_duty_cycle_us(us as u16, freq);
    }

    println!("duty: {}µs", us);
  }
  else {
    let duty = duty.clamp(0, 100) as u16;
    if channel == "a" {
      pwm.get_channel_a().set_duty_cycle_fraction(duty, 100).unwrap();
    }
    else {
      pwm.get_channel_b().set_duty_cycle_fraction(duty, 100).unwrap();
    }

    println!("duty: {}%", duty);
  }

  // End
  pwm.enable();

  Ok(())
}
