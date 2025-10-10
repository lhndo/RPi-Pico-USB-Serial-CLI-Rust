//! A Simple CLI Module

pub mod commands;
pub mod error;
pub mod parser;

pub use commands::CommandList;
pub use error::{Error, IntoTruncate, Result};
pub use parser::*;

use crate::device::Device as Context;
use crate::println;

pub use heapless::Vec;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              CLI
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct SimpleCli {
  command_list: CommandList,
}

impl SimpleCli {
  pub fn new(command_list: CommandList) -> Self {
    Self { command_list }
  }

  pub fn execute(&mut self, input: &str, context: &mut Context) -> Result<()> {
    // Extracting command name and list of arguments
    let (cmd_name, input_args) = input.split_once(' ').unwrap_or((input, ""));

    // Check if built-in help was called
    if cmd_name.is_empty() || cmd_name == "help" {
      self.built_in_help();
      return Ok(());
    }

    // Parsing arguments
    let cmd_args = parser::parse(input_args)?;

    // Execute Command
    let command = self.command_list.get_command(cmd_name)?;
    command.run(&cmd_args, context)
  }

  pub fn built_in_help(&self) {
    println!("\nAvailable Commands:");
    println!("-----------------------------");

    for command in self.command_list.commands.iter() {
      println!("{} - {}", command.name, command.desc);
    }
    println!("-----------------------------");
    println!("For more information type: command_name help\n");
  }
}
