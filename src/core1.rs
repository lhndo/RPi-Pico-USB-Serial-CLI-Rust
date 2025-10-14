//! Core 1 Main Loop
//!
//! Spawned by Device

#![allow(unused_mut)]

use crate::prelude::*;
use hal::multicore::Stack;

use rp2040_hal as hal;
//
use hal::{gpio, pac, sio, timer};

use heapless::mpmc::Queue;

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Globals
// —————————————————————————————————————————————————————————————————————————————————————————————————

// Memory Stack for core 1
pub static CORE1_STACK: Stack<2048> = Stack::new();

// Multicore MPMC Queue
pub static CORE1_QUEUE: Queue<Event, 8> = Queue::new();

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Core1 Main
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub fn core1_main(timer: timer::Timer) -> ! {
  // ————————————————————————————————————— Core 1 Boilerplate ————————————————————————————————————————

  let core = unsafe { pac::CorePeripherals::steal() };
  let mut pac = unsafe { pac::Peripherals::steal() };
  let mut sio = sio::Sio::new(pac.SIO);
  let pins = gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);
  let mut delay = cortex_m::delay::Delay::new(core.SYST, SYS_CLK_HZ.load(Ordering::Relaxed));
  let mut sio_fifo = sio.fifo;

  // ——————————————————————————————————————————— Pins ——————————————————————————————————————————————

  let mut test_input_pin: InputType = CONFIG.take_pin_by_alias("C1_IN_A").unwrap();
  let mut test_output_pin: OutputType = CONFIG.take_pin_by_alias("C1_OUT_A").unwrap();

  // Unsafe practice since we know that core0 also uses gpio25(LED)
  let mut led = pins.gpio25.into_push_pull_output();

  info!("Core 1 >> Initialised");

  // ————————————————————————————————————————— Main Loop ———————————————————————————————————————————

  loop {
    // ————————————————————————————————————————— Events ————————————————————————————————————————————

    while let Some(event) = CORE1_QUEUE.dequeue() {
      match event {
        Event::Blink { times, interval } => {
          blink_led(&mut led, &mut delay, times, interval);
        }
        Event::Sleep => {
          cortex_m::asm::wfi();
        }
      }
    }
    delay.delay_ms(10); // avoiding spinning in a tight loop
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Free Functions
// —————————————————————————————————————————————————————————————————————————————————————————————————

fn blink_led(led: &mut impl OutputPin, delay: &mut impl DelayMs<u32>, times: u16, interval: u16) {
  for _ in 0..times {
    let interval = interval as u32;
    led.set_high().ok();
    delay.delay_ms(interval);
    led.set_low().ok();
    delay.delay_ms(interval);
  }
}

// —————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Events
// —————————————————————————————————————————————————————————————————————————————————————————————————

pub enum Event {
  Blink { times: u16, interval: u16 },
  Sleep,
}
