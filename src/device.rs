//! Hardware Device Configuration
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Device
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::cell::RefCell;
use core::fmt::Write;

use crate::delay;
use crate::delay::DELAY;
use crate::serial_io;
use crate::serial_io::SERIAL;

use embedded_hal::pwm::SetDutyCycle;
use rp_pico as bsp;
use rp_pico::hal::Adc;
//
use bsp::hal;
use bsp::hal::Clock;
use bsp::hal::adc::{AdcPin, TempSense};
use bsp::hal::fugit::{Duration, ExtU32, MicrosDurationU32};
use bsp::hal::gpio;
use bsp::hal::timer::Alarm;
use bsp::hal::timer::Timer;
use bsp::hal::{clocks, pac, pac::interrupt, pwm, sio, timer, usb, watchdog};

use cortex_m::delay::Delay;
use cortex_m::interrupt::{Mutex, free};
use cortex_m::prelude::*;
use heapless::String;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

static ALARM_0: Mutex<RefCell<Option<timer::Alarm0>>> = Mutex::new(RefCell::new(None));

const INTERRUPT_0_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(10_000); // 100ms - 10hz

pub const SYS_CLK_HZ: u32 = 120_000_000;
pub const ADC_BITS: u32 = 12;
pub const ADC_MAX: f32 = ((1 << ADC_BITS) - 1) as f32;
pub const ADC_VREF: f32 = 3.3;
pub const PWM_TOP: u16 = u16::MAX; // Standard 16-bit resolution
pub const TEMP_SENSE_CHN: u8 = 255;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Device Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————
//
//WeAct Studio RP2040 - https://mischianti.org/weact-studio-rp2040-high-resolution-pinout-and-specs/
// https://mischianti.org/wp-content/uploads/2022/09/weact-studio-rp2040-raspberry-pi-pico-alternative-pinout-high-resolution.png
// RPi Pico - https://randomnerdtutorials.com/raspberry-pi-pico-w-pinout-gpios/
//             https://cdn-shop.adafruit.com/970x728/4864-04.png
//
// GPIO 29 - internal - ADC (ADC3) for measuring VSYS
// GPIO 25 - internal - LED
// GPIO 24 - WA extra GPIO / OG internal - Indicator for VBUS presence (high / low output)
// GPIO 23 - WA extra Button / OG -  Controls on-board SMPS (Switched Power Mode Supply)

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PinType {
  Input,
  Output,
  PwmOut,
  PWMIn,
  Adc,
}

// PWM
pub type Pwm1Type = pwm::Slice<pwm::Pwm1, pwm::FreeRunning>; // gpio 2 A
pub type Pwm2Type = pwm::Slice<pwm::Pwm2, pwm::FreeRunning>; // gpio 21 B
pub type Pwm3Type = pwm::Slice<pwm::Pwm3, pwm::FreeRunning>; // gpio 6 A

pub struct Pwms {
  pub pwm_1: Pwm1Type,
  pub pwm_2: Pwm2Type,
  pub pwm_3: Pwm3Type,
}

// ADC
pub type Adc0Type =
  AdcPin<gpio::Pin<gpio::bank0::Gpio26, gpio::FunctionSio<gpio::SioInput>, gpio::PullNone>>; // gpio 26
pub type Adc1Type =
  AdcPin<gpio::Pin<gpio::bank0::Gpio27, gpio::FunctionSio<gpio::SioInput>, gpio::PullNone>>; // gpio 27
pub type Adc2Type =
  AdcPin<gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionSio<gpio::SioInput>, gpio::PullNone>>; // gpio 28
pub type Adc3Type =
  AdcPin<gpio::Pin<gpio::bank0::Gpio29, gpio::FunctionSio<gpio::SioInput>, gpio::PullNone>>; // gpio 29

pub struct Acds {
  pub hal_adc:    Adc,
  pub temp_sense: TempSense,
  pub adc0:       Adc0Type,
  pub adc1:       Adc1Type,
  pub adc2:       Adc2Type,
  pub adc3:       Adc3Type,
}

impl Acds {
  /// One shot read of the ADC channel 0-3, and 255 (as TEMP_SENSE_CHN)
  pub fn read_channel(&mut self, id: u8) -> Option<u16> {
    match id {
      0 => self.hal_adc.read(&mut self.adc0).unwrap_or(None),
      1 => self.hal_adc.read(&mut self.adc1).unwrap_or(None),
      2 => self.hal_adc.read(&mut self.adc2).unwrap_or(None),
      3 => self.hal_adc.read(&mut self.adc3).unwrap_or(None),
      255 => self.hal_adc.read(&mut self.temp_sense).unwrap_or(None),
      _ => None,
    }
  }
}

// GPIO
// Inputs
pub type InputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioInput>, gpio::PullUp>;

pub struct Inputs {
  pub button: InputType, // internal 23
  pub input1: InputType, // gpio 22
  pub input2: InputType, // gpio 20
  pub input3: InputType, // gpio 9
}

pub type OutputType = gpio::Pin<gpio::DynPinId, gpio::FunctionSio<gpio::SioOutput>, gpio::PullDown>;

// Outputs
pub struct Outputs {
  pub led:     OutputType, // internal 25
  pub output1: OutputType, // gpio 0
  pub output2: OutputType, // gpio 1
  pub output3: OutputType, // gpio 3
}

// ———————————————————————————————————————————————————————————————————————————————————————————————
//                                         Device Struct
// ———————————————————————————————————————————————————————————————————————————————————————————————

pub struct Device {
  pub timer:    Timer,
  pub watchdog: watchdog::Watchdog,
  pub pwms:     Pwms,
  pub acds:     Acds,
  pub inputs:   Inputs,
  pub outputs:  Outputs,
}

impl Device {
  // ——————————————————————————————————————————— New ——————————————————————————————————————————————

  pub fn new() -> Self {
    //
    // ———————————————————————————————————— Hal Boilerplate ———————————————————————————————————————

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = sio::Sio::new(pac.SIO);
    let pac_pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    // ———————————————————————————————————————————————————————————————————————————————————————————
    //                                           Test todo
    // ———————————————————————————————————————————————————————————————————————————————————————————

    // pins = [
    //   [pac_pins.gpio0.get_input_override()]
    // ]

    // let a = pac_pins
    //   .gpio4
    //   .into_function::<gpio::FunctionPwm>()
    //   .into_pull_type::<gpio::PullNone>()
    //   .into_dyn_pin();

    // let a: gpio::Pin<gpio::DynPinId, gpio::FunctionPwm, gpio::PullNone> =
    //   pac_pins.gpio4.into_function();
    // let a = a.into_function::<gpio::FunctionSpi>();
    // let a = pac_pins.gpio4.into_dyn_pin();
    // let a = a.into_pull_type::<gpio::PullNone>();
    // if let Ok(p) = a.try_into_function::<gpio::FunctionPwm>() {
    //   let a = p;
    // };

    // ————————————————————————————————————————— Clocks ———————————————————————————————————————————

    let sys_clocks = clocks::init_clocks_and_plls(
      bsp::XOSC_CRYSTAL_FREQ, // 12Mhz
      pac.XOSC,
      pac.CLOCKS,
      pac.PLL_SYS,
      pac.PLL_USB,
      &mut pac.RESETS,
      &mut watchdog,
    )
    .ok()
    .unwrap();

    // ————————————————————————————————————————— Timer ————————————————————————————————————————————

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &sys_clocks);

    // ————————————————————————————————————————— Delay  ————————————————————————————————————————————

    let delay = Delay::new(core.SYST, sys_clocks.system_clock.freq().to_Hz());
    delay::init(delay); // initializing global DELAY

    // ———————————————————————————————————————— USB Bus ———————————————————————————————————————————

    // usb bus used to create serial and usb_device then into >> serialio
    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
      pac.USBCTRL_REGS,
      pac.USBCTRL_DPRAM,
      sys_clocks.usb_clock,
      true,
      &mut pac.RESETS,
    ));

    // quick persistent singleton creation
    let usb_bus_ref = cortex_m::singleton!(: UsbBusAllocator<usb::UsbBus> = usb_bus).unwrap();

    DELAY.us(200);

    // ————————————————————————————————————— Serial Device ————————————————————————————————————————

    let serial = SerialPort::new(usb_bus_ref); // Needs to be set before usb_dev

    // ——————————————————————————————————————— Usb Device —————————————————————————————————————————

    let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
      .strings(&[StringDescriptors::default()
        .manufacturer("LH_Eng")
        .product("embedded_serial_cli")
        .serial_number("TEST")])
      .unwrap()
      .device_class(usbd_serial::USB_CLASS_CDC)
      .build();

    // ————————————————————————————————————— SERIAL Handle ————————————————————————————————————————

    // Init SERIAL global
    serial_io::init(serial, usb_dev);

    // ————————————————————————————————————— USB Interrupt ————————————————————————————————————————

    // Disabling USB interrupt due to Fault/Bug and keeping polling into IRQ_0
    // This bug  happens even if we blink a simple led in a loop and send a msg though serial

    // // Enable the USB interrupt
    // unsafe {
    //   pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    // };

    // Priming USB connection
    SERIAL.poll_usb();

    // ————————————————————————————————————————— Interrupts ———————————————————————————————————————

    let mut alarm0 = timer.alarm_0().unwrap();
    alarm0.schedule(INTERRUPT_0_US).unwrap();
    alarm0.enable_interrupt();

    free(|cs| {
      ALARM_0.borrow(cs).borrow_mut().replace(alarm0);
    });

    // Enable Interrupt
    unsafe {
      pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    // —————————————————————————————————————————— ADC —————————————————————————————————————————————

    let mut hal_adc = hal::Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks
    let temp_sense = hal_adc.take_temp_sensor().unwrap();

    let adc0 = AdcPin::new(pac_pins.gpio26.into_floating_input()).unwrap();
    let adc1 = AdcPin::new(pac_pins.gpio27.into_floating_input()).unwrap();
    let adc2 = AdcPin::new(pac_pins.gpio28.into_floating_input()).unwrap();
    let adc3 = AdcPin::new(pac_pins.gpio29.into_floating_input()).unwrap();

    let acds = Acds {
      hal_adc,
      temp_sense,
      adc0,
      adc1,
      adc2,
      adc3,
    };

    // —————————————————————————————————————————— PWM —————————————————————————————————————————————

    // PWM Setup Macro
    macro_rules! setup_pwm {
      ($pwm_slices:ident, $slice_num:ident, $channel:ident, $pin:expr, $hz:expr) => {{
        let mut slice = $pwm_slices.$slice_num;

        slice.disable();
        slice.set_top(PWM_TOP);
        let (int, frac) = calculate_pwm_dividers($hz, PWM_TOP, false);
        slice.set_div_int(int);
        slice.set_div_frac(frac);
        slice.$channel.set_duty_cycle_percent(50).unwrap();
        slice.$channel.output_to($pin);
        slice.enable();

        slice
      }};
    }

    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    let pwm_1: pwm::Slice<pwm::Pwm1, pwm::FreeRunning> =
      setup_pwm!(pwm_slices, pwm1, channel_a, pac_pins.gpio2, 50.0);
    let pwm_2 = setup_pwm!(pwm_slices, pwm2, channel_b, pac_pins.gpio21, 50.0);
    let pwm_3 = setup_pwm!(pwm_slices, pwm3, channel_a, pac_pins.gpio6, 50.0);

    let pwms = Pwms { pwm_1, pwm_2, pwm_3 };

    // —————————————————————————————————————————— Pins ————————————————————————————————————————————

    // Inputs
    let button: InputType = pac_pins.gpio23.into_pull_up_input().into_dyn_pin();
    let input1: InputType = pac_pins.gpio20.into_pull_up_input().into_dyn_pin();
    let input2: InputType = pac_pins.gpio22.into_pull_up_input().into_dyn_pin();
    let input3: InputType = pac_pins.gpio9.into_pull_up_input().into_dyn_pin();

    let inputs = Inputs {
      button,
      input1,
      input2,
      input3,
    };

    // Outputs
    let led: OutputType = pac_pins.gpio25.into_push_pull_output().into_dyn_pin();
    let output1: OutputType = pac_pins.gpio0.into_push_pull_output().into_dyn_pin();
    let output2: OutputType = pac_pins.gpio1.into_push_pull_output().into_dyn_pin();
    let output3: OutputType = pac_pins.gpio3.into_push_pull_output().into_dyn_pin();

    let outputs = Outputs {
      led,
      output1,
      output2,
      output3,
    };

    // —————————————————————————————————————— Construct ———————————————————————————————————————————

    Self {
      timer,
      watchdog,
      pwms,
      acds,
      inputs,
      outputs,
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        Extension Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ————————————————————————————————————————— Timer Ext ————————————————————————————————————————————

pub trait TimerExt {
  fn now(&self) -> Duration<u64, 1, 1_000_000>;
  fn print_time(&self) -> String<16>;
  fn delay_ms(&self, millis: u32);
  fn delay_us(&self, us: u32);
}

impl TimerExt for Timer {
  fn now(&self) -> Duration<u64, 1, 1_000_000> {
    self.get_counter().duration_since_epoch()
  }

  fn print_time(&self) -> String<16> {
    let total_micros = self.now().to_micros();

    // Calculate components
    let total_millis = total_micros / 1_000;
    let total_seconds = total_millis / 1_000;
    let min = total_seconds / 60;
    let sec = total_seconds % 60;
    let mil = total_millis % 1_000;
    let mic = total_micros % 1_000;

    // Use heapless::String for formatting
    let mut time: String<16> = String::new();
    write!(&mut time, "{min}:{sec:02}.{mil:03}.{mic:03}").expect("print time fmt");
    time
  }

  /// Count Down Delay ms - Precise and async-friendly
  fn delay_ms(&self, millis: u32) {
    let mut count_down = self.count_down();
    count_down.start(millis.millis());
    let _ = nb::block!(count_down.wait());
  }

  /// Count Down Delay us
  fn delay_us(&self, us: u32) {
    let mut count_down = self.count_down();
    count_down.start(us.micros());
    let _ = nb::block!(count_down.wait());
  }
}

// ————————————————————————————————————————— Adc Tools ———————————————————————————————————————————

pub trait AdcConversion {
  /// Convert raw u16 ADC reading to volts.
  fn to_voltage(&self) -> f32;
  fn to_resistance(&self, ref_res: u32) -> f32;
}

// Impl for u16, assuming 12-bit ADC (0..=4095) and 3.3 V reference.
impl AdcConversion for u16 {
  fn to_voltage(&self) -> f32 {
    (*self as f32) * ADC_VREF / ADC_MAX
  }

  fn to_resistance(&self, ref_res: u32) -> f32 {
    let x: f32 = (ADC_MAX / *self as f32) - 1.0;
    // ref_res / x // If you ref resistor to Gnd instead of V+
    ref_res as f32 * x
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          Free Functions
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Reset to USB Flash mode
pub fn device_reset_to_usb() {
  unsafe {
    rp2040_rom::ROM::reset_usb_boot(0, 0);
  }
}

/// Reset device
pub fn device_reset() {
  cortex_m::peripheral::SCB::sys_reset();
}

/// Calculates pwm int and frac clock dividers  based on sys clock, top, and desired hz frequency
pub fn calculate_pwm_dividers(hz: f32, top: u16, phase_correct: bool) -> (u8, u8) {
  let hz = if phase_correct { hz * 2.0 } else { hz };
  let divider = SYS_CLK_HZ as f32 / (hz * (top as f32 + 1.0));
  let clamped_divider = divider.clamp(1.0, 255.9375);

  let div_int = (clamped_divider + 0.5) as u8;
  let div_frac = ((clamped_divider - div_int as f32) * 16.0 + 0.5) as u8;

  (div_int, div_frac)
}

pub fn calculate_pwm_dividers_simple(hz: f32) -> (u8, u8) {
  calculate_pwm_dividers(hz, PWM_TOP, false)
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Interrupts
// ————————————————————————————————————————————————————————————————————————————————————————————————

// Interrupt 0
#[pac::interrupt]
fn TIMER_IRQ_0() {
  SERIAL.poll_usb();
  SERIAL.update_connected_status();
  // Reset interrupt timer safely
  free(|cs| {
    if let Some(alarm) = ALARM_0.borrow(cs).borrow_mut().as_mut() {
      alarm.clear_interrupt();
      alarm.schedule(INTERRUPT_0_US).unwrap();
    };
  })
}

// Disabling USB interrupt due to Fault/Bug and keeping polling into IRQ_0
// This bug happens even if we blink a simple led in a loop and send a msg though serial

// // Polling the USB device to keep the connection alive even if we stall
// #[pac::interrupt]
// fn USBCTRL_IRQ() {
//   SERIAL.poll_usb();
//   // SERIAL.update_connected();
// }
