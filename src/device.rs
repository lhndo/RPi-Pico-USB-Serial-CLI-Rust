//! Hardware Device Configuration
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Device
// ————————————————————————————————————————————————————————————————————————————————————————————————

//
// RPi Pico           - https://cdn-shop.adafruit.com/970x728/4864-04.png
// WeAct Studio RP2040 - https://mischianti.org/wp-content/uploads/2022/09/weact-studio-rp2040-raspberry-pi-pico-alternative-pinout-high-resolution.png
//
// GPIO 29 - WA extra GPIO (Analog) / RP Pico internal - ADC (ADC3) for measuring VSYS
// GPIO 25 - Internal LED
// GPIO 24 - WA extra GPIO / RP Pico internal - Indicator for VBUS presence (high / low output)
// GPIO 23 - WA extra Button / RP Pico -  Controls on-board SMPS (Switched Power Mode Supply)

use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::adcs::Acds;
use crate::delay;
use crate::delay::DELAY;
use crate::gpios::{InputType, IoPins, OutputType};
use crate::pwms::Pwms;
use crate::serial_io;
use crate::serial_io::SERIAL;
use crate::state::State;

use rp_pico::hal;
use rp_pico::hal::Clock;
use rp_pico::hal::adc::AdcPin;
use rp_pico::hal::fugit::{Duration, ExtU32, MicrosDurationU32};
use rp_pico::hal::gpio::{self};
use rp_pico::hal::timer::Alarm;
use rp_pico::hal::timer::Timer;
use rp_pico::hal::{clocks, pac, pac::interrupt, pwm, sio, timer, usb, watchdog};

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

// Pin Aliases
pub const LED: usize = 25;
pub const BUTTON: usize = 23; // built-in button on WeAct RP2040

pub static SYS_CLK_HZ: AtomicU32 = AtomicU32::new(0);

static ALARM_0: Mutex<RefCell<Option<timer::Alarm0>>> = Mutex::new(RefCell::new(None));
const INTERRUPT_0_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(10_000); // 10ms - 100hz

// ———————————————————————————————————————————————————————————————————————————————————————————————
//                                         Device Struct
// ———————————————————————————————————————————————————————————————————————————————————————————————

pub struct Device {
  pub timer:    Timer,
  pub watchdog: watchdog::Watchdog,
  pub pwms:     Pwms,
  pub acds:     Acds,
  pub inputs:   IoPins<InputType>,
  pub outputs:  IoPins<OutputType>,
  pub state:    State,
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
    let pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    // ————————————————————————————————————————— Clocks ———————————————————————————————————————————

    let sys_clocks = clocks::init_clocks_and_plls(
      rp_pico::XOSC_CRYSTAL_FREQ, // 12Mhz
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

    let delay = Delay::new(core.SYST, sys_clocks.system_clock.freq().to_Hz());

    // Init DELAY Global
    delay::init(delay);

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

    // Small pause for initialisation
    DELAY.us(200);

    // ————————————————————————————————————— Serial Device ————————————————————————————————————————

    // SerialPort needs to be created before UsbDev and requires a reference to the UsbBus
    let serial_port = SerialPort::new(usb_bus);

    // ——————————————————————————————————————— Usb Device —————————————————————————————————————————

    // Usb Device creation using the UsbBus
    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
      .strings(&[StringDescriptors::default()
        .manufacturer("LH_Eng")
        .product("embedded_serial_cli")
        .serial_number("TEST")])
      .unwrap()
      .device_class(usbd_serial::USB_CLASS_CDC)
      .build();

    // ————————————————————————————————————— SERIAL Handle ————————————————————————————————————————

    // Init SERIAL Global
    // This is the main interface for interacting with the Serial and the Usb Device
    serial_io::init(serial_port, usb_dev);
    SERIAL.poll_usb();

    // ————————————————————————————————————————— Interrupts ———————————————————————————————————————

    // ————————————————————————————————————— USB Interrupt ————————————————————————————————————————

    // Enabling the USB interrupt
    unsafe {
      pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);
    };

    // Priming USB otherwise connection is not established with the hosts
    SERIAL.poll_usb();

    // ———————————————————————————————————————— Alarm 0 ———————————————————————————————————————————

    // Creating an interrupt for keeping the Usb connection alive by polling every 10ms
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

    // The hal_adc is the main gateway for interracting with the ADC
    // We store it in device.acds.hal_adc
    let mut hal_adc = hal::Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks
    let temp_sense = hal_adc.take_temp_sensor().unwrap();

    // Analog pins assigned to the Adc channels
    let adc0 = AdcPin::new(pins.gpio26).unwrap();
    let adc1 = AdcPin::new(pins.gpio27).unwrap();
    let adc2 = AdcPin::new(pins.gpio28).unwrap();
    let adc3 = AdcPin::new(pins.gpio29).unwrap();

    let acds = Acds {
      hal_adc,
      temp_sense,
      adc0,
      adc1,
      adc2,
      adc3,
    };

    // —————————————————————————————————————————— PWM —————————————————————————————————————————————

    let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // device.pwms is the main interface for the pwm channels
    let mut pwms = Pwms::new(pwm_slices, 50);

    // Assigning PWM pins to the appropriate channels
    pwms.pwm2.get_channel_b().output_to(pins.gpio21);
    pwms.pwm3.get_channel_a().output_to(pins.gpio6);
    pwms.pwm4.get_channel_a().output_to(pins.gpio8);

    // ———————————————————————————————————————— GP Pins ———————————————————————————————————————————

    // Digital - General Purpose Inputs
    let input_pins: [InputType; _] = [
      pins.gpio9.reconfigure().into_dyn_pin(),
      pins.gpio20.reconfigure().into_dyn_pin(),
      pins.gpio22.reconfigure().into_dyn_pin(),
      pins.gpio23.reconfigure().into_dyn_pin(), // BUTTON on WeAct RP2040
    ];

    // Digital - General Purpose Outputs
    let output_pins: [OutputType; _] = [
      pins.gpio0.reconfigure().into_dyn_pin(),
      pins.gpio1.reconfigure().into_dyn_pin(),
      pins.gpio3.reconfigure().into_dyn_pin(),
      pins.gpio25.reconfigure().into_dyn_pin(), // LED
    ];

    let inputs = IoPins::new(input_pins);
    let outputs = IoPins::new(output_pins);

    // ————————————————————————————————————————— State ————————————————————————————————————————————

    let state = State::new();

    // —————————————————————————————————————— Construct ———————————————————————————————————————————

    Self {
      timer,
      watchdog,
      pwms,
      acds,
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
  fn print_time(&self) -> String<16>;
  fn delay_ms(&self, millis: u32);
  fn delay_us(&self, us: u32);
}

/// Timer extension that provides extra utilities such as a better delay implementation
/// Access them though device.timer
impl TimerExt for Timer {
  fn now(&self) -> Duration<u64, 1, 1_000_000> {
    self.get_counter().duration_since_epoch()
  }

  /// Printing Time
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

/// Interrupt 0
#[pac::interrupt]
fn TIMER_IRQ_0() {
  // Do something here in a timed interrupt

  // Reset interrupt timer safely
  free(|cs| {
    if let Some(alarm) = ALARM_0.borrow(cs).borrow_mut().as_mut() {
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

  // All explicit reads are done in CS blocks in the main program loops
  // We search the rx buffer for an interrupt character and flush the rest
  SERIAL.poll_for_interrupt_char();
}
