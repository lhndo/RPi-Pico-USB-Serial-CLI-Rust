// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Device
// ————————————————————————————————————————————————————————————————————————————————————————————————

#![allow(static_mut_refs)]

use crate::delay::DELAY;
use crate::serial_io::Serialio;
use crate::serial_io::SERIAL;

use core::fmt::Write;
use heapless::String;

use rp_pico as bsp;
//
use bsp::hal;
use bsp::hal::adc::{AdcPin, TempSense};
use bsp::hal::fugit::{Duration, ExtU32, MicrosDurationU32};
use bsp::hal::timer::Alarm;
use bsp::hal::timer::Timer;
use bsp::hal::Clock;
use bsp::hal::{clocks, pac, pac::interrupt, pwm, sio, timer, usb, watchdog};

use cortex_m::delay::Delay;
use cortex_m::prelude::*;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::SerialPort;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

static mut ALARM: Option<timer::Alarm0> = None; //  USB Interrupt Timer
const USB_INTERRUPT_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(10_000);

pub const ADC_BITS: u32 = 12;
pub const ADC_MAX: f32 = ((1 << ADC_BITS) - 1) as f32;
pub const ADC_VREF: f32 = 3.3;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Device Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub type Pwm = pwm::Channel<pwm::Slice<pwm::Pwm7, pwm::FreeRunning>, pwm::B>;

pub struct Device {
  pub timer:    Timer,
  pub watchdog: watchdog::Watchdog,
  pub adc:      hal::Adc,
  pub pwm:      Pwm,
  pub pins:     ConfiguredPins,
}

impl Device {
  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                            New
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  pub fn new() -> Self {
    //
    // ———————————————————————————————————— Hal Boilerplate ———————————————————————————————————————

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
    let sio = sio::Sio::new(pac.SIO);
    let pac_pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

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

    // —————————————————————————————————————————— ADC —————————————————————————————————————————————

    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks
    let temp_sense = adc.take_temp_sensor().unwrap();

    // ————————————————————————————————————————— Timer ————————————————————————————————————————————

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &sys_clocks);

    // ————————————————————————————————————————— Delay  ————————————————————————————————————————————

    let delay = Delay::new(core.SYST, sys_clocks.system_clock.freq().to_Hz());
    DELAY.init(delay);

    // ———————————————————————————————————————— USB Bus ———————————————————————————————————————————

    static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBus>> = None;

    let usb_bus_alloc = UsbBusAllocator::new(usb::UsbBus::new(
      pac.USBCTRL_REGS,
      pac.USBCTRL_DPRAM,
      sys_clocks.usb_clock,
      true,
      &mut pac.RESETS,
    ));

    unsafe { USB_BUS.replace(usb_bus_alloc) };
    let usb_bus_ref = unsafe { USB_BUS.as_ref().unwrap() };
    DELAY.delay_us(200);

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

    let serialio = Serialio::new(serial, usb_dev);
    SERIAL.init(serialio);

    // ——————————————————————————————————— USB Poll Interrupt —————————————————————————————————————

    let mut alarm0 = timer.alarm_0().unwrap();
    alarm0.schedule(USB_INTERRUPT_US).unwrap();
    alarm0.enable_interrupt();
    unsafe {
      pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
      ALARM.replace(alarm0);
    }

    // —————————————————————————————————————————— PWM —————————————————————————————————————————————

    // PWM
    // Usage: pwm.set_duty(65535_u16);
    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Channel B - pwm7  for GPIO 15;
    let mut pwm_slice_7 = pwm_slices.pwm7;
    pwm_slice_7.set_ph_correct();
    pwm_slice_7.set_div_int(20u8); // 50 hz
    pwm_slice_7.enable();

    let mut pwm = pwm_slice_7.channel_b;
    pwm.output_to(pac_pins.pwm_pin); // Pin defined in DevicePins Struct

    // —————————————————————————————————————————— Pins ————————————————————————————————————————————

    let pins = ConfiguredPins {
      led: pac_pins.led.reconfigure(),
      button: pac_pins.button.reconfigure(),
      adc_pin: AdcPin::new(pac_pins.adc_pin.reconfigure()).unwrap(),
      temp_sense,
    };

    // —————————————————————————————————————————— Self ————————————————————————————————————————————

    Self {
      timer,
      watchdog,
      adc,
      pwm,
      pins,
    }
  }
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                        DevicePins Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

hal::bsp_pins!(
  Gpio25 {
    name: led,
    aliases: {
      FunctionSioOutput, PullDown: Led
        // Function, PullType: Type Alias
        // Examples:
        // FunctionSioInput, PullUp: LedSioIPU,
        // FunctionUart, PullNone: LedUart1Cts,
        // FunctionSpi, PullNone: LedSpi0Sck,
        // FunctionI2C, PullUp: LedI2C1Sda,
        // FunctionPwm, PullNone: LedPwm3A,
        // FunctionPio0, PullNone: LedPio0,
        // FunctionPio1, PullNone: Gp25Pio1
    }
},
Gpio23 {
    name: button,
    aliases: {
        FunctionSioInput, PullUp: Button
    }
},

Gpio29 {
  name: adc_pin,
  aliases: {
    FunctionSioInput, PullUp: AdcPin_
  }
},

Gpio15 {
  name: pwm_pin,
  aliases: {
    FunctionPwm, PullDown: PwmPin

  }
},

);


pub struct ConfiguredPins {
  pub led:        Led,
  pub button:     Button,
  pub adc_pin:    AdcPin<AdcPin_>,
  pub temp_sense: TempSense,
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                       Extension Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————


// ————————————————————————————————————————— Timer Ext ————————————————————————————————————————————
pub trait TimerExt {
  fn now(&self) -> Duration<u64, 1, 1_000_000>;
  fn print_time(&self) -> String<16>;
  fn delay_cd_ms(&self, millis: u32);
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

  /// Count Down Delay - Precise and async-friendly
  fn delay_cd_ms(&self, millis: u32) {
    let mut count_down = self.count_down();
    count_down.start(millis.millis());
    let _ = nb::block!(count_down.wait());
  }
}


// ————————————————————————————————————————— To Voltage ———————————————————————————————————————————

pub trait ToVoltage {
  /// Convert raw u16 ADC reading to volts.
  fn to_voltage(&self) -> f32;
}

// Impl for u16, assuming 12-bit ADC (0..=4095) and 3.3 V reference.
impl ToVoltage for u16 {
  fn to_voltage(&self) -> f32 {
    (*self as f32) * ADC_VREF / ADC_MAX
  }
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Free Functions
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

/// Polling the USB device to keep the connection alive even if we stall in main
/// SERIAL and USB methods use critical section, so this interrupt will not trigger
/// during their operations
#[pac::interrupt]
fn TIMER_IRQ_0() {
  SERIAL.poll_usb();

  // Reset interrupt timer
  critical_section::with(|cs| unsafe {
    if let Some(alarm) = ALARM.as_mut() {
      alarm.clear_interrupt();
      alarm.schedule(USB_INTERRUPT_US).unwrap();
    };
  })
}
