// #![allow(unused_imports)]

use crate::prelude::*;
use crate::simple_cli::*;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globasls
// ————————————————————————————————————————————————————————————————————————————————————————————————

type Result<T> = core::result::Result<T, CliError>;

// -----------------------------------------------------------------------------
//                              Commands Config
// -----------------------------------------------------------------------------

const NUM_COMMANDS: usize = 5;

pub const CMDS: [Command; NUM_COMMANDS] = [
  Command {
    name: "test",
    desc: "test | <arg:float> [opt:int] [on <true|false>] [path:string]",
    func: test,
  },
  Command {
    name: "blink",
    desc: "Blink Led | [times:int]",
    func: blink,
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
    desc: "Restart in USB Flash mode",
    func: flash,
  },
];

// -----------------------------------------------------------------------------
//                               Functions Config
// -----------------------------------------------------------------------------

// ———————————————————————————————————————————— Test ——————————————————————————————————————————————

fn test(args: &[Arg], device: &mut Device) -> Result<()> {
  println!("Running 'test': \n");

  let arg: f32 = get_parsed_param("arg", args)?;
  let opt: u8 = get_parsed_param("opt", args).unwrap_or(0); // With default
  let on: bool = get_parsed_param("on", args).unwrap_or(false);
  let path: &str = get_str_param("path", args).unwrap_or("");

  println!("arg = {arg}");
  println!("opt = {opt}");
  println!("on = {on}");
  println!("path = {path}");

  Ok(())
}

// —————————————————————————————————————————— Blink —————————————————————————————————————————————
// ex: blink times=4

fn blink(args: &[Arg], device: &mut Device) -> Result<()> {
  println!("Blinking Led!");

  let times: u8 = get_parsed_param("times", args).unwrap_or(10); // With 10 default


  for n in 1..(times + 1) {
    print!("Blink {} | ", n);
    device.pins.led.toggle().unwrap();
    device.timer.delay_cd_ms(400);
  }

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
  DELAY.delay_ms(1500); // Waiting for reset msg to appear
  device_reset();
  Ok(())
}

// ——————————————————————————————————————————— Flash ——————————————————————————————————————————————

fn flash(args: &[Arg], device: &mut Device) -> Result<()> {
  print!("\nRestarting in USB Flash mode!...\n");
  device_reset_to_usb();

  Ok(())
}
