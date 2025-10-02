//! Main device abstraction layer for the RP2040 microcontroller
//!
//! This module provides a unified `Device` struct that initializes and manages
//! all hardware peripherals including GPIO pins, PWM channels, ADCs, timers,
//! and system resources. It handles the low-level HAL boilerplate and presents
//! an organized interface for application code.
//!
//! The main Pin Configuration is done thought the setup_pins macro data structure

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Device
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::adcs::Adcs;
use crate::delay;
use crate::delay::DELAY;
use crate::gpios::{InputType, IoPins, OutputType};
use crate::pwms::Pwms;
use crate::serial_io;
use crate::serial_io::SERIAL;
use crate::state::State;
use crate::{build_pin_aliases, set_function_pins};

use critical_section::{Mutex, with};
use rp2040_hal as hal;
use rp2040_hal::gpio::{AnyPin, DynPullType};
//
use hal::Adc;
use hal::Clock;
use hal::fugit::{Duration, MicrosDurationU32};
use hal::pac::interrupt;
use hal::sio::SioFifo;
use hal::timer::{Alarm, Timer};
use hal::{clocks, gpio, pac, pwm, sio, timer, usb, watchdog};

use cortex_m::delay::Delay;

use heapless::String;
use pastey::paste;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Bootloader
// —————————————————————————————————————————————————————————————————————————————————————————————————
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Pin Definitions
// —————————————————————————————————————————————————————————————————————————————————————————————————

// RPi Pico           - https://pico.pinout.xyz
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

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Pins Setup
// —————————————————————————————————————————————————————————————————————————————————————————————————

// NA = Not Available
#[rustfmt::skip]
macro_rules! setup_pins {

  ($func:ident, $pins:ident, $obj:ident ) => { 
  set_function_pins!($func, $pins, $obj,
  
  //ADC  
  //Alias    GPIO  Func  Valid_Pins
  (ADC0,      26,  ADC) // GP26
  (ADC1,      27,  ADC) // GP27
  (ADC2,      28,  ADC) // GP28
  (ADC3,      29,  ADC) // GP29
  
  //PWM  
  (PWM0_A,    NA,  PWM) // GP0, GP16
  (PWM0_B,    NA,  PWM) // GP1, GP17
  (PWM1_A,    NA,  PWM) // GP2, GP18
  (PWM1_B,    NA,  PWM) // GP3, GP19
  (PWM2_A,    NA,  PWM) // GP4, GP20
  (PWM2_B,    21,  PWM) // GP5, GP21
  (PWM3_A,     6,  PWM) // GP6, GP22
  (PWM3_B,    NA,  PWM) // GP7
  (PWM4_A,     8,  PWM) // GP8
  (PWM4_B,    NA,  PWM) // GP9
  (PWM5_A,    NA,  PWM) // GP10, GP26
  (PWM5_B,    NA,  PWM) // GP11, GP27
  (PWM6_A,    NA,  PWM) // GP12, GP28
  (PWM6_B,    NA,  PWM) // GP13
  (PWM7_A,    NA,  PWM) // GP14
  (PWM7_B,    NA,  PWM) // GP15

  //I2C
  (I2C0_SDA,   2,  I2C) // GP0, GP4, GP8, GP12, GP16, GP20, GP28
  (I2C0_SCL,  NA,  I2C) // GP1, GP5, GP9, GP13, GP17, GP21

  (I2C1_SDA,  NA,  I2C) // GP2, GP6, GP10, GP14, GP18, GP22, GP26
  (I2C1_SCL,  NA,  I2C) // GP3, GP7, GP11, GP15, GP19, GP27

  //SPI
  (SPI0_RX,    4,  SPI) // GP0, GP4, GP16, GP20
  (SPI0_TX,   NA,  SPI) // GP3, GP19
  (SPI0_SCK,  NA,  SPI) // GP2, GP18, GP22
  (SPI0_CSN,  NA,  SPI) // GP1, GP5, GP17, GP21
  
  (SPI1_RX,   NA,  SPI) // GP8, GP12, GP28
  (SPI1_TX,   NA,  SPI) // GP7, GP11, GP15, GP27
  (SPI1_SCK,  NA,  SPI) // GP6, GP10, GP14, GP26
  (SPI1_CSN,  NA,  SPI) // GP9, GP13

  //UART
  (UART0_TX,   5,  UART) // GP0, GP12, GP16, GP28
  (UART0_RX,  NA,  UART) // GP1, GP13, GP17
  (UART0_CTS, NA,  UART) // GP2, GP14, GP18
  (UART0_RTS, NA,  UART) // GP3, GP15, GP19
  
  (UART1_TX,  NA,  UART) // GP4, GP8, GP20
  (UART1_RX,  NA,  UART) // GP5, GP9, GP21
  (UART1_CTS, NA,  UART) // GP6, GP10, GP22, GP26
  (UART1_RTS, NA,  UART) // GP7, GP11, GP27

  // Inputs
  (IN_A    ,   9  ,  IN)
  (IN_B    ,  20  ,  IN)
  (IN_C    ,  22  ,  IN)
  (BUTTON  ,  23  ,  IN)
  
  // Ouputs
  (OUT_A   ,   0  , OUT)
  (OUT_B   ,   1  , OUT)
  (OUT_C   ,   3  , OUT)
  (LED     ,  25  , OUT)

  );}
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // 12Mhz
const DEFAULT_PWM_FREQUENCY: u32 = 50; //hz

pub static SYS_CLK_HZ: AtomicU32 = AtomicU32::new(0);

// Interrupts
static ALARM_0: Mutex<RefCell<Option<timer::Alarm0>>> = Mutex::new(RefCell::new(None));
const INTERRUPT_0_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(10_000); // 10ms - 100hz

// GPIO Pin ID Aliases. E.g. PinID::LED = 25 as usize
pub struct PinID;
setup_pins!(ALIASES, None, PinID);

pub type DynPinType = gpio::Pin<gpio::DynPinId, gpio::DynFunction, gpio::DynPullType>;

// ———————————————————————————————————————————————————————————————————————————————————————————————
//                                         Device Struct
// ———————————————————————————————————————————————————————————————————————————————————————————————

pub struct Device {
  pub sio_fifo: SioFifo,
  pub timer:    Timer,
  pub watchdog: watchdog::Watchdog,
  pub pwms:     Pwms,
  pub adcs:     Adcs,
  pub inputs:   IoPins<InputType>,
  pub outputs:  IoPins<OutputType>,
  pub state:    State,
}

impl Device {
  pub fn new() -> Self {
    // ———————————————————————————————————— Hal Boilerplate ———————————————————————————————————————

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = sio::Sio::new(pac.SIO);
    let pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);
    let sio_fifo = sio.fifo;

    // ————————————————————————————————————————— Clocks ———————————————————————————————————————————

    let sys_clocks = clocks::init_clocks_and_plls(
      XOSC_CRYSTAL_FREQ,
      pac.XOSC,
      pac.CLOCKS,
      pac.PLL_SYS,
      pac.PLL_USB,
      &mut pac.RESETS,
      &mut watchdog,
    )
    .ok()
    .unwrap();

    let sys_clk_hz: u32 = sys_clocks.system_clock.freq().to_Hz();
    SYS_CLK_HZ.store(sys_clk_hz, Ordering::Relaxed);

    // ————————————————————————————————————————— Timer ————————————————————————————————————————————

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &sys_clocks);

    // ————————————————————————————————————————— Delay  ————————————————————————————————————————————

    let delay = Delay::new(core.SYST, sys_clk_hz);
    delay::init(delay); // Init DELAY Global

    // ———————————————————————————————————————— USB Bus ———————————————————————————————————————————

    // UsbBus used for creation of Serial and UsbDevice
    let usb_bus_alloc = UsbBusAllocator::new(usb::UsbBus::new(
      pac.USBCTRL_REGS,
      pac.USBCTRL_DPRAM,
      sys_clocks.usb_clock,
      true,
      &mut pac.RESETS,
    ));

    // Storing UsbBus into a singleton and getting a mutable reference
    let usb_bus = cortex_m::singleton!(: UsbBusAllocator<usb::UsbBus> = usb_bus_alloc).unwrap();
    DELAY.us(200); // Small pause for initialisation

    // ————————————————————————————————————— Serial Port ————————————————————————————————————————

    // SerialPort needs to be created before UsbDev and requires a reference to the UsbBus
    let serial_port = SerialPort::new(usb_bus);

    // ——————————————————————————————————————— Usb Device —————————————————————————————————————————

    // Usb Device creation using the UsbBus
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
      .strings(&[StringDescriptors::default()
        .manufacturer("LH Eng")
        .product("Rpi Pico - USB Serial CLI")
        .serial_number("0000")])
      .unwrap()
      .device_class(usbd_serial::USB_CLASS_CDC)
      .build();

    // ————————————————————————————————————————— SERIAL ————————————————————————————————————————————

    // Init SERIAL Global - main interface for interacting with the Serial and the Usb Device
    serial_io::init(serial_port, usb_dev);

    // —————————————————————————————————————————— ADC —————————————————————————————————————————————

    // The hal_adc (device.adcs.hal_adc) is the main interface for interracting with the ADC
    let mut hal_adc = Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks
    let temp_sense = hal_adc.take_temp_sensor().unwrap();

    // Initialise ADC pins from setup_pins by calling e.g. adcs.set_adc0(pins.gpio26); ...
    let mut adcs = Adcs::new(hal_adc, temp_sense);
    setup_pins!(ADC, pins, adcs);

    // —————————————————————————————————————————— PWM —————————————————————————————————————————————

    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Initialise PWM pins from setup_pins by calling e.g. pwms.set_pwm2_b(pins.gpio21); ...
    let mut pwms = Pwms::new(pwm_slices, sys_clk_hz, DEFAULT_PWM_FREQUENCY);
    setup_pins!(PWM, pins, pwms);

    // ———————————————————————————————————— Extra Function Pins ———————————————————————————————————

    // Sets pin variables, e.g. let I2C0_SDA = pins.gpio0. Use these to build interfaces.
    setup_pins!(I2C, pins, None);
    setup_pins!(SPI, pins, None);
    setup_pins!(UART, pins, None);

    // ———————————————————————————————————————— GP Pins ———————————————————————————————————————————

    // Initialise GPIO pins i.e. inputs.add_pin(pins.gpio23, 23)
    let mut inputs = IoPins::<InputType>::new();
    let mut outputs = IoPins::<OutputType>::new();
    setup_pins!(GPIO_IN, pins, inputs);
    setup_pins!(GPIO_OUT, pins, outputs);

    // ————————————————————————————————————————— Interrupts ———————————————————————————————————————

    // ALARM0 interrupt setup
    let mut alarm0 = timer.alarm_0().unwrap();
    alarm0.schedule(INTERRUPT_0_US).unwrap();
    alarm0.enable_interrupt();
    with(|cs| {
      ALARM_0.borrow(cs).replace(Some(alarm0));
    });

    // Enabling IRQ 0 - ALARM0
    unsafe {
      pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    // Enabling the USB IRQ
    unsafe {
      pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
    };

    // ————————————————————————————————————————— State ————————————————————————————————————————————

    let state = State::new();

    // —————————————————————————————————————— Construct ———————————————————————————————————————————

    Self {
      sio_fifo,
      timer,
      watchdog,
      pwms,
      adcs,
      inputs,
      outputs,
      state,
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Extension Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ————————————————————————————————————————— Timer Ext ————————————————————————————————————————————

pub trait TimerExt {
  fn now(&self) -> Duration<u64, 1, 1_000_000>;
  fn print_time(&self) -> String<32>;
}

/// Timer extension that provides extra utilities such as a better delay implementation
/// Access them though device.timer
impl TimerExt for Timer {
  fn now(&self) -> Duration<u64, 1, 1_000_000> {
    self.get_counter().duration_since_epoch()
  }

  /// Printing Time with formatted units
  fn print_time(&self) -> String<32> {
    let total_us = self.now().to_micros();

    // Isolate the second and sub-second components
    let total_secs = total_us / 1_000_000;
    let sub_s_us = total_us % 1_000_000;

    // Calculate each unit from the components
    let hr = total_secs / 3600;
    let min = (total_secs % 3600) / 60;
    let sec = total_secs % 60;
    let mil = sub_s_us / 1_000;
    let mic = sub_s_us % 1_000;

    // Format into a heapless String with unit labels
    let mut time: String<32> = String::new();
    write!(&mut time, "{hr}h {min:02}m {sec:02}s {mil:03}ms {mic:03}us").unwrap();
    time
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          Free Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Reset to USB Flash mode
pub fn device_reset_to_usb() {
  rp2040_hal::rom_data::reset_to_usb_boot(0, 0);
}

/// Reset device
pub fn device_reset() {
  cortex_m::peripheral::SCB::sys_reset();
}

/// Converts concrete pin into a fully dynamic pin
pub fn pin_into_full_dynamic<P: AnyPin>(pin: P) -> DynPinType {
  let pin: gpio::SpecificPin<P> = pin.into();
  pin.into_dyn_pin().into_function().into_pull_type::<DynPullType>()
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Interrupts
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Interrupt 0
#[pac::interrupt]
fn TIMER_IRQ_0() {
  // Do something here in a timed interrupt

  // Reset interrupt timer safely
  with(|cs| {
    if let Some(alarm) = ALARM_0.borrow_ref_mut(cs).as_mut() {
      alarm.clear_interrupt();
      alarm.schedule(INTERRUPT_0_US).unwrap();
    };
  })
}

/// USB Interrupt
/// Polling the USB device to keep the connection alive even if we stall
#[pac::interrupt]
fn USBCTRL_IRQ() {
  SERIAL.poll_usb();

  // We search the rx buffer for an interrupt character and flush the rest
  // This is ok since all explicit reads are done in CS blocks in the main program loop
  // If we don't read the data, the interrupt will keep firing freezing the device
  SERIAL.poll_for_interrupt_cmd();
}
