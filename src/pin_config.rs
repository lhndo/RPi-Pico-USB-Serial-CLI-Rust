use crate::system::config::Def;
use crate::system::config::Group::*;
use crate::system::config::PinId::*;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Reference
// —————————————————————————————————————————————————————————————————————————————————————————————————

// RPi Pico            - https://pico.pinout.xyz
// WeAct Studio RP2040 - https://mischianti.org/wp-content/uploads/2022/09/weact-studio-rp2040-raspberry-pi-pico-alternative-pinout-high-resolution.png
//
//                                                     --RPi Pico--
//                                                      ___USB___
// (PWM0 A)(UART0  TX)(I2C0 SDA)(SPI0  RX)   GP0  |  1 |o       o| 40 | VBUS 5V
// (PWM0 B)(UART0  RX)(I2C0 SCL)(SPI0 CSn)   GP1  |  2 |o       o| 39 | VSYS 5V*
//                                           GND  |  3 |o       o| 38 | GND
// (PWM1 A)(UART0 CTS)(I2C1 SDA)(SPI0 SCK)   GP2  |  4 |o       o| 37 | 3V3  En
// (PWM1 B)(UART0 RTS)(I2C1 SCL)(SPI0  TX)   GP3  |  5 |o       o| 36 | 3V3  Out
// (PWM2 A)(UART1  TX)(I2C0 SDA)(SPI0  RX)   GP4  |  6 |o       o| 35 | ADC  VREF
// (PWM2 B)(UART1  RX)(I2C0 SCL)(SPI0 CSn)   GP5  |  7 |o       o| 34 | GP28 A2    (SPI1  RX)(I2C0 SDA)(UART0  TX)(PWM6 A)
//                                           GND  |  8 |o       o| 33 | ADC  GND
// (PWM3 A)(UART1 CTS)(I2C1 SDA)(SPI1 SCK)   GP6  |  9 |o       o| 32 | GP27 A1    (SPI1  TX)(I2C1 SCL)(UART1 RTS)(PWM5 B)
// (PWM3 B)(UART1 RTS)(I2C1 SCL)(SPI1  TX)   GP7  | 10 |o       o| 31 | GP26 A0    (SPI1 SCK)(I2C1 SDA)(UART1 CTS)(PWM5 A)
// (PWM4 A)(UART1  TX)(I2C0 SDA)(SPI1  RX)   GP8  | 11 |o       o| 30 | RUN
// (PWM4 B)(UART1  RX)(I2C0 SCL)(SPI1 CSn)   GP9  | 12 |o       o| 29 | GP22       (SPI0 SCK)(I2C1 SDA)(UART1 CTS)(PWM3 A)
//                                           GND  | 13 |o       o| 28 | GND
// (PWM5 A)(UART1 CTS)(I2C1 SDA)(SPI1 SCK)   GP10 | 14 |o       o| 27 | GP21       (SPI0 CSn)(I2C0 SCL)(UART1  RX)(PWM2 B)
// (PWM5 B)(UART1 RTS)(I2C1 SCL)(SPI1  TX)   GP11 | 15 |o       o| 26 | GP20       (SPI0  RX)(I2C0 SDA)(UART1  TX)(PWM2 A)
// (PWM6 A)(UART0  TX)(I2C0 SDA)(SPI1  RX)   GP12 | 16 |o       o| 25 | GP19       (SPI0  TX)(I2C1 SCL)(UART0 RTS)(PWM1 B)
// (PWM6 B)(UART0  RX)(I2C0 SCL)(SPI1 CSn)   GP13 | 17 |o       o| 24 | GP18       (SPI0 SCK)(I2C1 SDA)(UART0 CTS)(PWM1 A)
//                                           GND  | 18 |o       o| 23 | GND
// (PWM7 A)(UART0 CTS)(I2C1 SDA)(SPI1 SCK)   GP14 | 19 |o       o| 22 | GP17       (SPI0 CSn)(I2C0 SCL)(UART0  RX)(PWM0 B)
// (PWM7 B)(UART0 RTS)(I2C1 SCL)(SPI1  TX)   GP15 | 20 |o__ooo__o| 21 | GP16       (SPI0  RX)(I2C0 SDA)(UART0  TX)(PWM0 A)
//
//                                             --[ SWD: CLK, GND, DIO ]--
//
// | Pin     | Description           | Notes                                                                                  |
// |---------|-----------------------|----------------------------------------------------------------------------------------|
// | VSYS*   | System voltage in/out | 5V out when powered by USB (diode to VBUS), 1.8V to 5.5V in if powered externally      |
// | 3V3 Out | Chip 3V3 supply       | Can be used to power external circuitry, recommended to keep the load less than 300mA  |
// | GP23    | RT6150B-33GQW P-Select| LOW (def) high efficiency (PFM), HIGH improved ripple (PWM)  | WeAct - extra Button    |
// | GP24    | VBUS Sense            | Detect USB power or VBUS pin                                 | Weact - extra GPIO      |
// | GP25    | User LED              |                                                                                        |
// | GP29 A3 | VSYS Sense            | Read VSYS/3 through resistor divider and FET Q1              | WeAct - extra GPIO A3   |
// | A4      | Temperature           | Read onboard temperature sensor                                                        |
//

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Pin Configuration
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[rustfmt::skip]
pub const PIN_DEFINITION: &[Def] = {
    &[
        //           Alias       GPIO            Group           Valid Pins
        // Core0 ————————————————————————————————————————————————————————————
        
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
        
        // Other
        Def { alias: "DHT22",    id: Gpio(16), group: Other   },

        //           Alias       GPIO            Group           Valid Pins
        // Core1 ————————————————————————————————————————————————————————————
        // Try defining Core1 Aliases with a C1 prefix and define them as C1 groups

        // Inputs
        Def { alias: "C1_IN_A",    id: Gpio(10),  group: C1_Inputs  },

        // Ouputs 
        Def { alias: "C1_OUT_A",   id: Gpio(11),  group: C1_Outputs },
        
    ]
};
