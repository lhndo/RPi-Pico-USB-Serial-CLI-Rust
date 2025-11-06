//! Main device abstraction layer for the RP2040 microcontroller
//!
//! This module provides a unified `Device` struct that initializes and manages
//! all hardware peripherals including GPIO pins, PWM channels, ADCs, timers,
//! and system resources. It handles the low-level HAL boilerplate and presents
//! an organized interface for application code.
//!
//! The main Pin Configuration is done thought config.rs

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Device
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};

use super::adcs::Adcs;
use super::config::{self, CONFIG};
use super::delay;
use super::delay::DELAY;
use super::gpios::{InputType, IoPins, OutputType};
use super::pwms::Pwms;
use super::serial_io::{self, SERIAL};

use crate::drivers::dht22::DHT22;
use crate::state::State;
use crate::{gpio, main_core1};

use rp2040_hal as hal;
//
use hal::fugit::{Duration, MicrosDurationU32};
use hal::multicore::Multicore;
use hal::pac::interrupt;
use hal::sio::SioFifo;
use hal::timer::{Alarm, Timer};
use hal::watchdog::Watchdog;
use hal::{Adc, Clock, clocks, gpio, pac, pwm, sio, timer, usb, watchdog};

use cortex_m::delay::Delay;
use critical_section::{Mutex, with};
use heapless::String;
use heapless::mpmc::Queue;
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Bootloader
// —————————————————————————————————————————————————————————————————————————————————————————————————

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // 12Mhz
const DEFAULT_PWM_FREQUENCY: u32 = 50; //hz

pub static SYS_CLK_HZ: AtomicU32 = AtomicU32::new(0);

// Multicore MPMC Queue
pub static CORE0_QUEUE: Queue<EventCore0, 8> = Queue::new();

// Interrupts
static ALARM_0: Mutex<RefCell<Option<timer::Alarm0>>> = Mutex::new(RefCell::new(None));
const INTERRUPT_0_US: MicrosDurationU32 = MicrosDurationU32::from_ticks(100_000); // 100ms - 10hz

// ———————————————————————————————————————————————————————————————————————————————————————————————
//                                             Device
// ———————————————————————————————————————————————————————————————————————————————————————————————

pub struct Device {
    pub sio_fifo: SioFifo,
    pub timer:    Timer,
    pub watchdog: Watchdog,
    pub pwms:     Pwms,
    pub adcs:     Adcs,
    pub inputs:   IoPins<InputType>,
    pub outputs:  IoPins<OutputType>,
    pub state:    State,
    pub dht:      DHT22,
}

impl Device {
    pub fn new() -> Self {
        // ———————————————————————————————————— Hal Boilerplate ———————————————————————————————————————

        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();
        let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
        let sio = sio::Sio::new(pac.SIO);
        let pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);
        let mut sio_fifo = sio.fifo;

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
        sio_fifo.drain();

        // ————————————————————————————————————————— Timer ————————————————————————————————————————————

        let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &sys_clocks);

        // ————————————————————————————————————————— Delay  ————————————————————————————————————————————

        let delay = Delay::new(core.SYST, sys_clk_hz);
        delay::init(delay); // Init DELAY Global

        // ————————————————————————————————————————— Core 1 ————————————————————————————————————————————

        let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio_fifo);
        let cores = mc.cores();
        let core1 = &mut cores[1];
        let _task = core1
            .spawn(main_core1::CORE1_STACK.take().unwrap(), move || main_core1::main_core1(timer));

        // ———————————————————————————————————————— USB Bus ———————————————————————————————————————————

        // UsbBus used for the creation of the Serial and UsbDevice
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

        // SerialPort has to be created before UsbDev and requires a reference to UsbBus
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

        // Creating and initializing the hal ADC
        let hal_adc = Adc::new(pac.ADC, &mut pac.RESETS); // Needs to be set after clocks
        let mut adcs = Adcs::new(hal_adc);

        for id in CONFIG.get_group_iter(config::Group::Adc) {
            let pin = CONFIG.take_pin(id).unwrap();
            adcs.register(pin);
        }

        // —————————————————————————————————————————— PWM —————————————————————————————————————————————

        let pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);
        let mut pwms = Pwms::new(pwm_slices, sys_clk_hz, DEFAULT_PWM_FREQUENCY);

        for id in CONFIG.get_group_iter(config::Group::Pwm) {
            let pin = CONFIG.take_pin(id).unwrap();
            pwms.register(pin);
        }

        // ———————————————————————————————————— Extra Function Pins ———————————————————————————————————

        // SPI, I2C, UART, etc

        // ———————————————————————————————————————— GP Pins ———————————————————————————————————————————

        let mut inputs = IoPins::<InputType>::new();
        let mut outputs = IoPins::<OutputType>::new();

        for id in CONFIG.get_group_iter(config::Group::Inputs) {
            let pin = CONFIG.take_pin(id).unwrap();
            inputs.register(pin);
        }

        for id in CONFIG.get_group_iter(config::Group::Outputs) {
            let pin = CONFIG.take_pin(id).unwrap();
            outputs.register(pin);
        }

        // —————————————————————————————————— DHT22 Temp Sensor ————————————————————————————————————

        let dht_pin: OutputType = CONFIG.take_pin(gpio!(DHT22)).unwrap();
        let dht = DHT22::new(dht_pin, timer);

        // ————————————————————————————————————— Interrupts ————————————————————————————————————————

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
            dht,
        }
    }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Events
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub enum EventCore0 {
    Done,
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

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Interrupts
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Interrupt 0
#[pac::interrupt]
fn TIMER_IRQ_0() {
    {
        // Do something here in a timed interrupt
    }

    // Reset interrupt timer
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
    // If we don't read the data, the interrupt will cause an interrupt storm freezing the device.
    SERIAL.poll_for_interrupt_cmd();
}
