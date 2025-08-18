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

use rp_pico as bsp;
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
use cortex_m::prelude::*;
use critical_section::Mutex;
use heapless::String;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

static ALARM: Mutex<RefCell<Option<timer::Alarm0>>> = Mutex::new(RefCell::new(None));

const USB_INTERRUPT_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(10_000);

pub const ADC_BITS: u32 = 12;
pub const ADC_MAX: f32 = ((1 << ADC_BITS) - 1) as f32;
pub const ADC_VREF: f32 = 3.3;

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

// PWM
pub type PwmAType = pwm::Channel<pwm::Slice<pwm::Pwm1, pwm::FreeRunning>, pwm::A>; // gpio 2
pub type PwmBType = pwm::Channel<pwm::Slice<pwm::Pwm3, pwm::FreeRunning>, pwm::A>; // gpio 6
pub type PwmCType = pwm::Channel<pwm::Slice<pwm::Pwm5, pwm::FreeRunning>, pwm::B>; // gpio 11
pub type PwmDType = pwm::Channel<pwm::Slice<pwm::Pwm2, pwm::FreeRunning>, pwm::B>; // gpio 21

pub struct Pwms {
  pub pwm_a: PwmAType,
  pub pwm_b: PwmBType,
  pub pwm_c: PwmCType,
  pub pwm_d: PwmDType,
}

// ADC
pub type Adc0Type = AdcPin<gpio::Pin<gpio::bank0::Gpio26, gpio::FunctionNull, gpio::PullDown>>; // gpio 26
pub type Adc1Type = AdcPin<gpio::Pin<gpio::bank0::Gpio27, gpio::FunctionNull, gpio::PullDown>>; // gpio 27
pub type Adc2Type = AdcPin<gpio::Pin<gpio::bank0::Gpio28, gpio::FunctionNull, gpio::PullDown>>; // gpio 28
pub type Adc3Type = AdcPin<gpio::Pin<gpio::bank0::Gpio29, gpio::FunctionNull, gpio::PullDown>>; // gpio 29

pub struct Acds {
  pub adc0: Adc0Type,
  pub adc1: Adc1Type,
  pub adc2: Adc2Type,
  pub adc3: Adc3Type,
  pub acd4_temp_sense: TempSense,
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

// Global Device
pub struct Device {
  pub timer:    Timer,
  pub watchdog: watchdog::Watchdog,
  pub hal_adc:  hal::Adc,
  pub pwms:     Pwms,
  pub acds:     Acds,
  pub inputs:   Inputs,
  pub outputs:  Outputs,
}

impl Device {
  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                           New
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  pub fn new() -> Self {
    //
    // ———————————————————————————————————— Hal Boilerplate ———————————————————————————————————————

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = sio::Sio::new(pac.SIO);
    let pac_pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

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
        .serial_number("static")])
      .unwrap()
      .device_class(usbd_serial::USB_CLASS_CDC)
      .build();

    // ————————————————————————————————————— SERIAL Handle ————————————————————————————————————————

    serial_io::init(serial, usb_dev);

    // ——————————————————————————————————— USB Poll Interrupt —————————————————————————————————————

    let mut alarm0 = timer.alarm_0().unwrap();
    alarm0.schedule(USB_INTERRUPT_US).unwrap();
    alarm0.enable_interrupt();

    critical_section::with(|cs| {
      ALARM.borrow(cs).borrow_mut().replace(alarm0);
    });

    unsafe {
      pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    // —————————————————————————————————————————— ADC —————————————————————————————————————————————

    let mut hal_adc = hal::Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks

    let adc0 = AdcPin::new(pac_pins.gpio26.reconfigure()).unwrap();
    let adc1 = AdcPin::new(pac_pins.gpio27.reconfigure()).unwrap();
    let adc2 = AdcPin::new(pac_pins.gpio28.reconfigure()).unwrap();
    let adc3 = AdcPin::new(pac_pins.gpio29.reconfigure()).unwrap();

    let temp_sense_adc4 = hal_adc.take_temp_sensor().unwrap();

    let acds = Acds {
      adc0,
      adc1,
      adc2,
      adc3,
      acd4_temp_sense: temp_sense_adc4,
    };

    // —————————————————————————————————————————— PWM —————————————————————————————————————————————

    // PWM Setup Macro
    macro_rules! setup_pwm {
      ($pwm_slices:ident, $slice_num:ident, $channel:ident, $pin:expr, $hz:expr) => {{
        let mut slice = $pwm_slices.$slice_num;
        let div_int = (125_000_000 / ($hz as u64 * 131072)) as u8;

        slice.set_ph_correct();
        slice.set_div_int(div_int);
        slice.enable();

        let mut channel = slice.$channel;
        channel.output_to($pin);

        channel
      }};
    }

    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    let pwm_a = setup_pwm!(pwm_slices, pwm1, channel_a, pac_pins.gpio2, 50);
    let pwm_b = setup_pwm!(pwm_slices, pwm3, channel_a, pac_pins.gpio6, 50);
    let pwm_c = setup_pwm!(pwm_slices, pwm5, channel_b, pac_pins.gpio11, 50);
    let pwm_d = setup_pwm!(pwm_slices, pwm2, channel_b, pac_pins.gpio21, 50);

    let pwms = Pwms {
      pwm_a,
      pwm_b,
      pwm_c,
      pwm_d,
    };

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
      hal_adc,
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

pub trait AdcTools {
  /// Convert raw u16 ADC reading to volts.
  fn to_voltage(&self) -> f32;
  fn to_resistance(&self, ref_res: u32) -> f32;
}

// Impl for u16, assuming 12-bit ADC (0..=4095) and 3.3 V reference.
impl AdcTools for u16 {
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

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Interrupts
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Polling the USB device to keep the connection alive even if we stall
/// SERIAL and USB methods use critical section, so this interrupt will not trigger during their operations
#[pac::interrupt]
fn TIMER_IRQ_0() {
  SERIAL.poll_usb();

  // Reset interrupt timer safely
  critical_section::with(|cs| {
    if let Some(alarm) = ALARM.borrow(cs).borrow_mut().as_mut() {
      alarm.clear_interrupt();
      alarm.schedule(USB_INTERRUPT_US).unwrap();
    };
  })
}
