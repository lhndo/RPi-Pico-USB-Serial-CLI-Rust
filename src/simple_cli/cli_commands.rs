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

const NUM_COMMANDS: usize = 3;

pub const CMDS: [Command; NUM_COMMANDS] = [
  Command {
    name: "test",
    desc: "tests | <arg:float> [opt:int] [on <true|false>] [path:string]",
    func: test,
  },
  Command {
    name: "another",
    desc: "Another command that does nothing",
    func: another,
  },
  Command {
    name: "help",
    desc: "Show command help",
    func: help,
  },
];

// -----------------------------------------------------------------------------
//                               Functions Config
// -----------------------------------------------------------------------------

// ---------------------------------- Test -------------------------------------

fn test(args: &[Arg]) -> Result<()> {
  println!("Running |test|: \n");

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

// --------------------------------- Another -----------------------------------

fn another(_args: &[Arg]) -> Result<()> {
  println!("This is a another command!");
  Ok(())
}

// ---------------------------------- Help -------------------------------------

fn help(_args: &[Arg]) -> Result<()> {
  println!("Available commands:\n");
  for cmd in CMDS.iter() {
    println!(" {} - {} ", &cmd.name, &cmd.desc);
  }
  println!("  exit - Exit program");
  Ok(())
}
