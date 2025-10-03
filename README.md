## Note:

* The intent is to develop an easy prototyping framework by using the **Raspberry Pi Pico** over **Serial USB** though a **CLI**
* This project is in an early WIP stage and I'm using it for learning and experimenting with Rust/Embedded development







## Setup:

* Clone this repository
* Connect the Pico in `BOOTSEL` mode and execute `cargo run` (*assuming you have the rust toolchain installed*)
* Connect to the Pico though a COM port using a **Serial Monitor tool** and follow the instructions

<br>

### Probe-rs

* For **probe-rs** with a **swd** interface you can flash/debug by running: `cargo embed-d` or `cargo embed default -- --no-default-features --features defmt`

* VS Code debug/tasks are also available




## Commands

* **CLI commands** are implemented in **simple_cli/commands/**

### Blink Example

```rust

pub fn build_blink_cmd() -> Command {
  Command {
    name: "blink",
    desc: "Blinks Onboard Led",
    help: "blink [times=10] [interval=200(ms)] [help]",
    func: blink_cmd,
  }
}

pub fn blink_cmd(cmd: &Command, args: &[Arguments], device: &mut Device) -> Result<()> {
  // Print Help
  if args.contains_param("help") {
    cmd.print_help();
    return Ok(());
  }

  let times: u16 = args.get_parsed_param("times").unwrap_or(10); // 10 default
  let interval: u16 = args.get_parsed_param("interval").unwrap_or(200); // 200ms default
  blink(device, times, interval)
}

// Separating functions from commands for stand alone use
pub fn blink(device: &mut Device, times: u16, interval: u16) -> Result<()> {
  println!("---- Blinking Led! ----");
  let led_id = CONFIG.get_id("LED").unwrap();
  let led = device.outputs.get_by_gpio_id(led_id).unwrap();
  let blink = 1;

  for n in 1..=times {
    print!("Blink {} | ", n);
    led.set_high().unwrap();
    device.timer.delay_ms(interval);
    led.set_low().unwrap();
    device.timer.delay_ms(interval);
  }

  Ok(())
}

```

## Configuration 

* The pin definition is set in **config.rs**

* The pins are dynamically built and assigned for the GPIO, PWM, ADC functions though **device.rs**.

* The "**device**" is set up in **device.rs** and encapsulated in a **Device** struct which is then borrowed to various **CLI** **commands**/**programs** 

### Configuration Example

```rust
static PIN_DEFINITION: &[Def] = {
    &[
        //           Alias       GPIO            Group           Valid Pins
        // ADC
        Def { alias: "ADC0",     id: Gpio(26), group: Adc    }, // GP26
        Def { alias: "ADC1",     id: Gpio(27), group: Adc    }, // GP27
        Def { alias: "ADC2",     id: Gpio(28), group: Adc    }, // GP28
        Def { alias: "ADC3",     id: Gpio(29), group: Adc    }, // GP29

        // PWM
        Def { alias: "PWM0_A",   id: NA,       group: Pwm    }, // GP0, GP16
        Def { alias: "PWM0_B",   id: NA,       group: Pwm    }, // GP1, GP17
        Def { alias: "PWM1_A",   id: NA,       group: Pwm    }, // GP2, GP18
        Def { alias: "PWM1_B",   id: NA,       group: Pwm    }, // GP3, GP19
        Def { alias: "PWM2_A",   id: NA,       group: Pwm    }, // GP4, GP20
        Def { alias: "PWM2_B",   id: Gpio(21), group: Pwm    }, // GP5, GP21s
        Def { alias: "PWM3_A",   id: Gpio(6),  group: Pwm    }, // GP6, GP22
        Def { alias: "PWM3_B",   id: NA,       group: Pwm    }, // GP7
        Def { alias: "PWM4_A",   id: Gpio(8),  group: Pwm    }, // GP8
        Def { alias: "PWM4_B",   id: NA,       group: Pwm    }, // GP9
        Def { alias: "PWM5_A",   id: NA,       group: Pwm    }, // GP10, GP26
        Def { alias: "PWM5_B",   id: NA,       group: Pwm    }, // GP11, GP27
        Def { alias: "PWM6_A",   id: NA,       group: Pwm    }, // GP12, GP28
        Def { alias: "PWM6_B",   id: NA,       group: Pwm    }, // GP13
        Def { alias: "PWM7_A",   id: NA,       group: Pwm    }, // GP14
        Def { alias: "PWM7_B",   id: NA,       group: Pwm    }, // GP15

        // I2C
        Def { alias: "I2C0_SDA", id: Gpio(2),  group: I2c    }, // GP0, GP4, GP8, GP12, GP16, GP20, GP28
        Def { alias: "I2C0_SCL", id: NA,       group: I2c    }, // GP1, GP5, GP9, GP13, GP17, GP21
        Def { alias: "I2C1_SDA", id: NA,       group: I2c    }, // GP2, GP6, GP10, GP14, GP18, GP22, GP26
        Def { alias: "I2C1_SCL", id: NA,       group: I2c    }, // GP3, GP7, GP11, GP15, GP19, GP27

        // SPI
        Def { alias: "SPI0_RX",  id: Gpio(4),  group: Spi    }, // GP0, GP4, GP16, GP20
        Def { alias: "SPI0_TX",  id: NA,       group: Spi    }, // GP3, GP19
        Def { alias: "SPI0_SCK", id: NA,       group: Spi    }, // GP2, GP18, GP22
        Def { alias: "SPI0_CSN", id: NA,       group: Spi    }, // GP1, GP5, GP17, GP21

        Def { alias: "SPI1_RX",  id: NA,       group: Spi    }, // GP8, GP12, GP28
        Def { alias: "SPI1_TX",  id: NA,       group: Spi    }, // GP7, GP11, GP15, GP27
        Def { alias: "SPI1_SCK", id: NA,       group: Spi    }, // GP6, GP10, GP14, GP26
        Def { alias: "SPI1_CSN", id: NA,       group: Spi    }, // GP9, GP13

        // UART
        Def { alias: "UART0_TX",  id: Gpio(5),  group: Uart  }, // GP0, GP12, GP16, GP28
        Def { alias: "UART0_CTS", id: NA,       group: Uart  }, // GP2, GP14, GP18
        Def { alias: "UART0_RX",  id: NA,       group: Uart  }, // GP1, GP13, GP17
        Def { alias: "UART0_RTS", id: NA,       group: Uart  }, // GP3, GP15, GP19
        
        Def { alias: "UART1_TX",  id: NA,       group: Uart  }, // GP4, GP8, GP20
        Def { alias: "UART1_RX",  id: NA,       group: Uart  }, // GP5, GP9, GP21
        Def { alias: "UART1_CTS", id: NA,       group: Uart  }, // GP6, GP10, GP22, GP26
        Def { alias: "UART1_RTS", id: NA,       group: Uart  }, // GP7, GP11, GP27

        // Inputs - Add your own aliases
        Def { alias: "IN_A",     id: Gpio(9),  group: Inputs  },
        Def { alias: "IN_B",     id: Gpio(20), group: Inputs  },
        Def { alias: "IN_C",     id: Gpio(22), group: Inputs  },
        Def { alias: "BUTTON",   id: Gpio(23), group: Inputs  },

        // Ouputs 
        Def { alias: "OUT_A",    id: Gpio(0),  group: Outputs },
        Def { alias: "OUT_B",    id: Gpio(1),  group: Outputs },
        Def { alias: "OUT_C",    id: Gpio(3),  group: Outputs },
        Def { alias: "LED",      id: Gpio(25), group: Outputs },
    ]
};
```
