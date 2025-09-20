pub mod core;
pub mod examples;

pub use super::*;
pub use core::*;
pub use examples::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                      Command List Builder
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn build_commands() -> CommandList {
  let mut command_list = CommandList::default();

  command_list.register_command(build_reset_cmd());
  command_list.register_command(build_flash_cmd());
  command_list.register_command(build_example_cmd());
  command_list.register_command(build_blink_cmd());
  command_list.register_command(build_read_adc_cmd());
  command_list.register_command(build_sample_adc_cmd());
  command_list.register_command(build_servo_cmd());
  command_list.register_command(build_set_pwm_cmd());
  command_list.register_command(build_panic_test_cmd());
  command_list.register_command(build_test_gpio_cmd());
  command_list.register_command(build_test_analog_cmd());

  command_list
}
