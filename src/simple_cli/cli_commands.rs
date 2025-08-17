//! CLI Commands/Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          CLI Commands
// ————————————————————————————————————————————————————————————————————————————————————————————————

// #![allow(unused_imports)]
use super::*;
use crate::prelude::*;

use embedded_hal::pwm::SetDutyCycle;


// -----------------------------------------------------------------------------
//                              Commands Config
// -----------------------------------------------------------------------------

const NUM_COMMANDS: usize = 7;

pub const CMDS: [Command; NUM_COMMANDS] = [
  Command {
    name: "test",
    desc: "test | <arg:float> [opt:int] [on <true|false>] [path:string]",
    func: test,
  },
  Command {
    name: "blink",
    desc: "Blink Onboard Led | [times]",
    func: blink,
  },
  Command {
    name: "servo",
    desc: "Set Servo PWM | [pause] [us]",
    func: servo,
  },
  Command {
    name: "read_adc",
    desc: "Read ADC | [ref_res]",
    func: read_adc,
  },
  Command {
    name: "help",
    desc: "Show command help",
    func: help,
  },
  Command {
    name: "reset",
    desc: "Reset device",
    func: reset,
  },
  Command {
    name: "flash",
    desc: "Restart device in USB Flash mode",
    func: flash,
  },
];

// -----------------------------------------------------------------------------
//                               Functions Config
// -----------------------------------------------------------------------------

type Result<T> = core::result::Result<T, CliError>;

// ———————————————————————————————————————————— Test ——————————————————————————————————————————————

fn test(args: &[Arg], device: &mut Device) -> Result<()> {
  let arg: f32 = get_parsed_param("arg", args)?;
  let opt: u8 = get_parsed_param("opt", args).unwrap_or(0); // With default
  let on: bool = get_parsed_param("on", args).unwrap_or(false);
  let path: &str = get_str_param("path", args).unwrap_or("");

  println!("Running 'test': \n");


  println!("arg = {arg}");
  println!("opt = {opt}");
  println!("on = {on}");
  println!("path = {path}");

  Ok(())
}

// —————————————————————————————————————————— Blink —————————————————————————————————————————————
// ex: blink times=4

fn blink(args: &[Arg], device: &mut Device) -> Result<()> {
  let times: u8 = get_parsed_param("times", args).unwrap_or(10); // 10 default

  println!("Blinking Led!");


  for n in 1..(times + 1) {
    print!("Blink {} | ", n);
    device.outputs.led.toggle().unwrap();
    device.timer.delay_ms(400);
  }

  Ok(())
}


// —————————————————————————————————————————— Servo —————————————————————————————————————————————
// ex: blink times=4

fn servo(args: &[Arg], device: &mut Device) -> Result<()> {
  let pause: u16 = get_parsed_param("pause", args).unwrap_or(2 * 1000); // 3s default
  let us: u16 = get_parsed_param("us", args).unwrap_or(1500); //  1500 us default

  const CYCLE: u16 = 20 * 1000; // 20ms - 50hz

  println!("---- Servo ----");

  let us = if CYCLE <= us { CYCLE } else { us };
  let pin = &mut device.pwms.pwm_a;

  print!("Setting PWM to: {}us, {}%  ... ", us, ((us as f32 / CYCLE as f32) * 100.0));
  pin.enable();
  pin.set_duty_cycle_fraction(us, CYCLE).unwrap();
  device.timer.delay_ms(pause as u32);
  pin.disable();
  println!("Done!");


  Ok(())
}


// —————————————————————————————————————————— Read ADC —————————————————————————————————————————————


fn read_adc(args: &[Arg], device: &mut Device) -> Result<()> {
  let ref_res: u32 = get_parsed_param("ref_res", args).unwrap_or(10_000); // 3s default

  println!("---- Read ADC ----");
  println!("Reference Pullup Resistor: {}ohm", ref_res);

  let channels_to_read: [u8; _] = [0, 1, 2, 3];

  for &channel in &channels_to_read {
    let adc_raw = match channel {
      0 => device.hal_adc.read(&mut device.acds.adc0).unwrap(),
      1 => device.hal_adc.read(&mut device.acds.adc1).unwrap(),
      2 => device.hal_adc.read(&mut device.acds.adc2).unwrap(),
      3 => device.hal_adc.read(&mut device.acds.adc3).unwrap(),
      _ => 0,
    };

    let adc_vol = adc_raw.to_voltage();
    let adc_res = adc_raw.to_resistance(ref_res);

    println!("ACD {}: {}, {:.2}V, {:.1}ohm ", channel, adc_raw, adc_vol, adc_res);
  }

  // read Temp Sense
  let adc_raw: u16 = device.hal_adc.read(&mut device.acds.acd4_temp_sense).unwrap();
  let adc_vol = adc_raw.to_voltage();
  let adc_res = adc_raw.to_resistance(ref_res);
  let sys_temp = 27.0 - (adc_raw.to_voltage() - 0.706) / 0.001721;
  println!("Temp Sense: {}, {:.2}V, {:.1}C", adc_raw, adc_vol, sys_temp);

  Ok(())
}

// ———————————————————————————————————————————— Help ——————————————————————————————————————————————

fn help(args: &[Arg], device: &mut Device) -> Result<()> {
  println!("Available commands:\n");
  for cmd in CMDS.iter() {
    println!(" {} - {} ", &cmd.name, &cmd.desc);
  }
  Ok(())
}

// ——————————————————————————————————————————— Reset ——————————————————————————————————————————————


fn reset(args: &[Arg], device: &mut Device) -> Result<()> {
  print!("\nResetting...\n");
  DELAY.ms(1500); // Waiting for reset msg to appear
  device_reset();
  Ok(())
}

// ——————————————————————————————————————————— Flash ——————————————————————————————————————————————

fn flash(args: &[Arg], device: &mut Device) -> Result<()> {
  print!("\nRestarting in USB Flash mode!...\n");
  device_reset_to_usb();

  Ok(())
}
