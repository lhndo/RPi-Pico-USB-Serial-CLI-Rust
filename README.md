## Note:
* This project is in an early WIP stage and I'm using it for learning and experimenting with Rust/Embedded development

* The intent is to develop an easy prototyping workflow by using the **Raspberry Pi Pico** over **Serial USB** though a **CL** like **interface**

* The "**device**" is set up in **device.rs** and encapsulated in a **Device** struct which is then borrowed to various **CLI** **commands**/**programs** 
* **CLI commands** are implemented in **simple_cli/commands/**

## Setup:

* Clone this repository
* Connect the Pico in `BOOTSEL` mode and execute `cargo run` (*assuming you have the rust toolchain installed*)
* Connect to the Pico though a COM port using a **Serial Monitor tool** and follow the instructions

<br>

* For `probe-rs` with a swd interface you can flash/debug by running: `cargo embed-d` or `cargo embed default -- --no-default-features --features defmt`

* VS Code debug/tasks are also available


## Command Example

### Blink

```rust

pub fn build_blink_cmd() -> Command {
  Command {
    name: "blink",
    desc: "Blinks Onboard Led",
    func: blink_cmd,
  }
}

pub fn blink_cmd_help() {
  println!("Help: blink");
  println!(
    "Blinks Onboard Led \n
    blink [times=10] [interval=200(ms)] [help]"
  )
}

pub fn blink_cmd(args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if contains_param("help", args) {
    blink_cmd_help();
    return Ok(());
  }

  let times: u16 = get_parsed_param("times", args).unwrap_or(10); // 10 default
  let interval: u16 = get_parsed_param("interval", args).unwrap_or(200); // 200 default
  blink(device, times, interval)
}

// Separating functions from commands for stand alone use
pub fn blink(device: &mut Device, times: u16, interval: u16) -> Result<()> {
  println!("---- Blinking Led! ----");
  let led = device.outputs.get_by_id(PinID::LED).unwrap();

  for n in 1..(times + 1) {
    print!("Blink {} | ", n);
    led.set_high().unwrap();
    device.timer.delay_ms(interval);
    led.set_low().unwrap();
    device.timer.delay_ms(interval);
  }

  Ok(())
}


```