//! Core commands
// Register new commands in commands.rs > Command List Builder

use super::*;
use crate::prelude::*;
use rp2040_hal::pwm;

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

pub fn build_pin_cmd() -> Command {
  Command {
    name: "pin",
    desc: "Read or Set the GPIO Pin State",
    help: "pin [alias=OUT_A(str)] / [gpio=..(u8)] [read(default)] [toggle] [high] [low] [help]",
    func: pin_cmd,
  }
}

pub fn pin_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  const DEFAULT_PIN: &str = "OUT_A";

  // Getting Alias or GPIO input -----------
  let alias = args.get_str_param("alias").unwrap_or(DEFAULT_PIN);
  let gpio = args.get_parsed_param::<u8>("gpio").ok();

  let (gpio, alias) = CONFIG.get_gpio_alias_pair(gpio, Some(alias))?;
  // -------------------------------------

  let toggle = args.contains_param("toggle");
  let high = args.contains_param("high");
  let low = args.contains_param("low");

  // Setting pin Mode
  if high || low || toggle {
    let pin = device.outputs.get(gpio)?;

    // Set mode
    if high {
      println!("> Output Pin: GPIO {gpio} - {alias}: set HIGH");
      pin.set_high().unwrap();
    }
    else if low {
      println!("> Output Pin: GPIO {gpio}: set LOW");
      pin.set_low().unwrap();
    }
    else if toggle {
      print!("> Output Pin: GPIO {gpio}: Toggled ");
      pin.toggle().unwrap();
      if pin.is_set_high().unwrap() {
        println!("HIGH")
      }
      else {
        println!("LOW")
      }
    }
  }
  // Reading Pin Mode
  // Input Pin Check
  else if let Ok(pin) = device.inputs.get(gpio) {
    println!(
      "> Input Pin: GPIO {gpio} - {alias}: {}",
      if pin.is_high().unwrap() { "HIGH" } else { "LOW" }
    )
  }
  // Output Pin Check
  else if let Ok(pin) = device.outputs.get(gpio) {
    println!(
      "> Output Pin: GPIO {gpio} - {alias}: {}",
      if pin.is_set_high().unwrap() { "HIGH" } else { "LOW" }
    )
  }
  else {
    return Err(Error::Configuration(ConfigError::GpioNotFound));
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

pub fn build_sample_adc_cmd() -> Command {
  Command {
    name: "sample_adc",
    desc: "Continuous sampling of an ADC channel",
    help: "sample_adc [alias=ADC0(str)] / [gpio=..(u8)] [ref_res=10000(ohm)] [interval=200(ms)] \
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

  const DEFAULT_PIN: &str = "ADC0";

  // Getting Alias or GPIO input ---------
  let alias = args.get_str_param("alias").unwrap_or(DEFAULT_PIN);
  let gpio = args.get_parsed_param::<u8>("gpio").ok();

  let (gpio, alias) = CONFIG.get_gpio_alias_pair(gpio, Some(alias))?;
  // -------------------------------------

  let ref_res: u32 = args.get_parsed_param("ref_res").unwrap_or(10_000);
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200);

  // Getting ADC channel based on pin number
  let channel = match gpio {
    26 => 0,
    27 => 1,
    28 => 2,
    29 => 3,
    255 => 4, // default TEMP_SENSE channel
    _ => return Err(Error::Configuration(ConfigError::OutOfBounds)),
  };

  println!("---- Sample ADC ----");
  println!("ADC Pin: GPIO {gpio} - {alias} | adc channel: {channel} |\n");
  println!("Reference Pullup Resistor: {}ohm", ref_res);
  println!("\nSend '~' to exit\n");

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

pub fn build_pwm_cmd() -> Command {
  Command {
    name: "pwm",
    desc: "Sets PWM  (defaults on GPIO 6 - PWM3A)",
    help:
      "pwm [alias=PWM2_B(str)] / [gpio=..(u8)] [freq=50(hz)] [duty=50(%)] [duty_us=..(us)] \n        \
       [top=-1(u16)] [phase=false(bool)] [disable=false(bool)] [help]",
    func: pwm_cmd,
  }
}

pub fn pwm_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  const DEFAULT_PIN: &str = "PWM2_B";

  // Getting Alias or GPIO input ---------
  let alias = args.get_str_param("alias").unwrap_or(DEFAULT_PIN);
  let gpio = args.get_parsed_param::<u8>("gpio").ok();

  let (gpio, alias) = CONFIG.get_gpio_alias_pair(gpio, Some(alias))?;
  // -------------------------------------

  let us: i32 = args.get_parsed_param("duty_us").unwrap_or(-1); //  -1 eq not set
  let duty: u8 = args.get_parsed_param("duty").unwrap_or(50); //  50% default
  let freq: u32 = args.get_parsed_param("freq").unwrap_or(50); // to_Hz
  let top: i32 = args.get_parsed_param("top").unwrap_or(-1); // 
  let phase: bool = args.get_parsed_param("phase").unwrap_or(false); // 
  let disable: bool = args.get_parsed_param("disable").unwrap_or(false); // false

  // Getting pwm information associated with the gpio pin
  let (slice_id, channel_type) = device.pwms.get_pwm_slice_id_by_gpio(gpio)?;

  // Print Pin information
  println!("Pwm Pin: GPIO {gpio} - {alias} | pwm: {slice_id}, channel: {channel_type} |\n");

  // Using a 'with' macro to be able to select the PWM slice
  // In regular usage you would call the pwm slice directly
  with_pwm_slice!(&mut device.pwms, slice_id, |pwm_slice| {
    pwm(pwm_slice, channel_type, us, duty, freq, top, phase, disable)
  })
}

#[allow(clippy::too_many_arguments)]
pub fn pwm<I>(
  pwm: &mut crate::pwms::PwmSlice<I>,
  channel: crate::pwms::Channel,
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
  print!("> Seting PWM : ");

  //
  if disable {
    pwm.disable();
    print!("Disabled |");
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

  // Getting pwm channel
  let mut channel = pwm.get_channel(channel);

  // Duty values for printing;
  let duty_us;
  let duty_p;

  // Set Duty
  if us > 0 {
    channel.set_duty_cycle_us(us as u16, freq);
    duty_us = us as u32;
    duty_p = (duty_us * freq + 5_000) / 10_000;
  }
  else {
    let duty = duty.clamp(0, 100) as u16;
    channel.set_duty_cycle_fraction(duty, 100).unwrap();
    duty_us = (duty as u32 * 10_000) / freq;
    duty_p = duty as u32;
  }

  let period_us: u32 = 1_000_000 / freq;

  println!(
    "freq: {freq}hz {period_us}us | duty: {duty_p}% {duty_us}µs | top: {top} | phase: {phase} |"
  );

  // End
  pwm.enable();

  Ok(())
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                               Log
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_log_cmd() -> Command {
  Command {
    name: "log",
    desc: "Sets the internal logging level",
    help: "log [level=\"\"(string)] [help] ",
    func: log_cmd,
  }
}

pub fn log_cmd(cmd: &Command, args: &[Argument], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }
  let level: &str = args.get_str_param("level").unwrap_or("");

  // Need if else for ignore case
  if level.eq_ignore_ascii_case("off") {
    LOG.set(LogLevel::Off)
  }
  else if level.eq_ignore_ascii_case("error") {
    LOG.set(LogLevel::Error)
  }
  else if level.eq_ignore_ascii_case("warn") {
    LOG.set(LogLevel::Warn)
  }
  else if level.eq_ignore_ascii_case("info") {
    LOG.set(LogLevel::Info)
  }
  else if level.eq_ignore_ascii_case("debug") {
    LOG.set(LogLevel::Debug)
  }
  else if level.eq_ignore_ascii_case("trace") {
    LOG.set(LogLevel::Trace)
  }
  else if !level.is_empty() {
    println!("Unknown level!\n Levels: off, error, warn, info, debug, trace\n")
  }

  println!("Log Level: {}", LOG.get());

  Ok(())
}
