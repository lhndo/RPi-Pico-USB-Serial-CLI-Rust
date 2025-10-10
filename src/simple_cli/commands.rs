//! Commands Module

pub mod base;
pub mod examples;

pub use base::*;
pub use examples::*;

pub use super::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

const MAX_CMDS: usize = 20;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                      Command List Builder
// —————————————————————————————————————————————————————————————————————————————————————————————————

/// Command List builder
/// Register new commands in the function below.
pub fn build_command_list() -> CommandList {
  let mut command_list = CommandList::default();

  // Base
  command_list.register_command(build_reset_cmd());
  command_list.register_command(build_flash_cmd());
  command_list.register_command(build_pin_cmd());
  command_list.register_command(build_read_adc_cmd());
  command_list.register_command(build_sample_adc_cmd());
  command_list.register_command(build_pwm_cmd());
  command_list.register_command(build_log_cmd());

  // Examples
  command_list.register_command(build_example_cmd());
  command_list.register_command(build_blink_cmd());
  command_list.register_command(build_servo_cmd());

  // Test
  command_list.register_command(build_test_gpio_cmd());
  command_list.register_command(build_test_analog_cmd());
  command_list.register_command(build_test_panic_cmd());
  command_list.register_command(build_test_log_cmd());

  command_list
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                          Command List
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[derive(Default, Debug)]
pub struct CommandList {
  pub commands: Vec<Command, MAX_CMDS>,
}

impl CommandList {
  pub fn register_command(&mut self, command: Command) {
    let _ = self.commands.push(command);
  }

  pub fn get_command(&self, name: &str) -> Result<&Command> {
    if let Some(cmd) = self.commands.iter().find(|cmd| cmd.name.eq_ignore_ascii_case(name)) {
      Ok(cmd)
    }
    else {
      Err(Error::CmdNotFound(name.into_truncated()))
    }
  }

  pub fn get_description(&self, cmd_name: &str) -> Result<&'static str> {
    let command = self.get_command(cmd_name)?;
    Ok(command.desc)
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                       Command Definition
// —————————————————————————————————————————————————————————————————————————————————————————————————

type FunctionCmd = fn(&Command, &[Argument], &mut Context) -> Result<()>;

#[derive(Debug)]
pub struct Command {
  pub name: &'static str,
  pub desc: &'static str,
  pub help: &'static str,
  pub func: FunctionCmd,
}

impl Command {
  pub fn run(&self, args: &[Argument], context: &mut Context) -> Result<()> {
    (self.func)(self, args, context)
  }

  pub fn print_help(&self) {
    println!("{}", self.desc);
    println!("{}", self.help)
  }

  pub fn print_description(&self) {
    println!("{}", self.desc);
  }
}
