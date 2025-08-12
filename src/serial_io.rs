// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Serial IO
// ————————————————————————————————————————————————————————————————————————————————————————————————
// This module owns the serial interface, the method of communicating outwards
// such as the usb device, and a write buffer


use crate::delay::DELAY;
use crate::fifo_buffer::FifoBuffer;

use core::cell::RefCell;
use core::fmt::Write;

use rp_pico as bsp;
//
use bsp::hal::usb::UsbBus;

use device_mod::UsbDevice;
use usb_device::class_prelude::*;
use usb_device::device as device_mod;
use usbd_serial::SerialPort;

use critical_section::Mutex;
use heapless::String;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub const WRITE_BUFF_SIZE: usize = 128;

pub static SERIAL: SerialHandle = SerialHandle;
pub static GLOBAL_SERIALIO: Mutex<RefCell<Option<Serialio>>> = Mutex::new(RefCell::new(None));


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                      SerialHandle Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————


/// Serial Handle for the GLOBAL SERIAL object
pub struct SerialHandle;

impl SerialHandle {
  pub fn init(&self, serial: Serialio) {
    critical_section::with(|cs| {
      GLOBAL_SERIALIO.borrow(cs).replace(Some(serial));
    });
  }


  /// Polls the USB device and returns true if data was exchanged.
  pub fn poll_usb(&self) -> bool {
    critical_section::with(|cs| {
      GLOBAL_SERIALIO.borrow_ref_mut(cs).as_mut().map(|s| s.poll_usb()).unwrap_or(false)
    })
  }

  #[inline(always)]
  pub fn get_serial_mutex(&self) -> &Mutex<RefCell<Option<Serialio>>> {
    &GLOBAL_SERIALIO
  }


  /// Checks if an interrupt command was received via the USB serial.
  pub fn poll_for_interrupt_cmd(&self) -> bool {
    critical_section::with(|cs| {
      GLOBAL_SERIALIO
        .borrow_ref_mut(cs)
        .as_mut()
        .map(|s| s.poll_for_interrupt_cmd())
        .unwrap_or(false)
    })
  }


  /// Flushes the write buffer in a blocking manner, sending data to the host.
  pub fn flush_write(&self) {
    critical_section::with(|cs| {
      if let Some(ref mut serialio) = GLOBAL_SERIALIO.borrow_ref_mut(cs).as_mut() {
        serialio.flush_write();
      }
    });
  }


  /// Reads a line from the USB serial into the provided buffer.
  /// Returns the number of bytes read (up to but not including the newline).
  pub fn read_line(&self, buffer: &mut [u8]) -> usize {
    critical_section::with(|cs| {
      GLOBAL_SERIALIO.borrow_ref_mut(cs).as_mut().map(|s| s.read_line(buffer)).unwrap_or(0)
    })
  }


  pub fn write(&self, data: &[u8]) -> Result<usize> {
    critical_section::with(|cs| {
      GLOBAL_SERIALIO
        .borrow(cs)
        .borrow_mut()
        .as_mut()
        .map(|s| s.write(data))
        .unwrap_or(Err(usb_device::UsbError::WouldBlock))
    })
  }
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Serial IO
// ————————————————————————————————————————————————————————————————————————————————————————————————


pub type SerialDev = SerialPort<'static, UsbBus>;
pub type UsbDev = UsbDevice<'static, UsbBus>;
pub type Result<T> = core::result::Result<T, UsbError>;


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Serialio Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Serialio {
  pub serial:       SerialDev,
  pub usb_dev:      UsbDev,
  pub write_buffer: FifoBuffer<WRITE_BUFF_SIZE>,
}


impl Serialio {
  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                               New
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  pub fn new(serial: SerialDev, usb_dev: UsbDev) -> Self {
    let write_buffer = FifoBuffer::<WRITE_BUFF_SIZE>::new();


    // —————————————————————————————————————————— Self ————————————————————————————————————————————

    Self {
      serial,
      usb_dev,
      write_buffer,
    }
  }


  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                           Methods
  // ——————————————————————————————————————————————————————————————————————————————————————————————


  /// Polls the usb device for rx tx data, and returns true if some data was exchanged.
  /// Must poll the usb for every 10ms to be compliant
  #[inline(always)]
  pub fn poll_usb(&mut self) -> bool {
    self.usb_dev.poll(&mut [&mut self.serial])
  }


  /// Polls serial read buffer for an excape character (Ctrl+C or 'c')
  /// Runs once, non interrupting. Returns true if found.
  /// To be used in loops that need to be interrupted from the command line
  #[inline(always)]
  pub fn poll_for_interrupt_cmd(&mut self) -> bool {
    let mut interrupt_received = false;

    if self.usb_dev.poll(&mut [&mut self.serial]) {
      let mut buffer = [0u8; 64];

      if let Ok(count) = self.serial.read(&mut buffer)
        && let Some(index) = buffer.iter().position(|&b| b == 0x03 || b == b'c')
      {
        interrupt_received = true;
      }
    }
    interrupt_received
  }


  /// Writes the byte slice and returns a Result(usize) or a blocking error
  /// Immediately polls the usb device and flush sends the entire write buffer
  #[inline(always)]
  fn write(&mut self, data: &[u8]) -> Result<usize> {
    let count = self.write_buffer.append(data).unwrap_or(0);

    if count == 0 {
      Err(UsbError::WouldBlock)
    } else {
      self.flush_write();
      Ok(count)
    }
  }


  /// Blocking write for clearing the write buffer by sending repeated packets to the host
  #[inline(always)]
  pub fn flush_write(&mut self) {
    while !self.write_buffer.is_empty() {
      if let Ok(count) = self.serial.write(self.write_buffer.data()) {
        self.write_buffer.pop(count)
      }
      self.poll_usb();
    }
  }


  /// Blocking read until a new line is received or the buffer is full.
  /// Remember to clear the read buffer beforehand and append the used count
  #[inline(always)]
  pub fn read_line(&mut self, buffer: &mut [u8]) -> usize {
    let max_len = buffer.len();
    let mut used = 0;

    loop {
      // Get a new usb package
      while !self.poll_usb() {
        DELAY.delay_us(5);
      }

      // Read as much as possible into the read buffer
      match self.serial.read(&mut buffer[used..]) {
        Ok(count) if count > 0 => {
          if let Some(index) = buffer[used..(used + count)].iter().position(|&b| b == b'\n') {
            used += index;
            break;
          }
          used += count;
          if used >= max_len {
            break;
          }
        }
        Ok(_) => {}
        Err(usb_device::UsbError::WouldBlock) => {}
        Err(_) => {
          break;
        }
      }
    }
    self.flush_read_all();
    used
  }


  /// Tries to flush the rest of the serial read into void. Not guaranteed
  /// it succeeds since we don't know if the host stopped sending messages
  #[inline(always)]
  fn flush_read_all(&mut self) {
    const MAX_EMPTY_READ_ATTEMPTS: i32 = 20;
    let mut temp_buffer = [0u8; 64];
    let mut retries = MAX_EMPTY_READ_ATTEMPTS; // Limit retries to avoid infinite loop

    while retries > 0 {
      self.poll_usb();
      DELAY.delay_us(5);

      match self.serial.read(&mut temp_buffer[..]) {
        Ok(count) if count > 0 => {
          retries = MAX_EMPTY_READ_ATTEMPTS; // Got data, reset retry count
        }
        Ok(_) => {
          retries -= 1; // No data read, decrement retries
        }
        Err(UsbError::WouldBlock) => {
          // No data available right now
          retries -= 1;
        }
        Err(_) => {
          break;
        }
      }
    }
  }
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————


// ——————————————————————————————————————————— Write ——————————————————————————————————————————————

type ResultWrite = core::result::Result<(), core::fmt::Error>;

impl Write for Serialio {
  #[inline(always)]
  fn write_str(&mut self, s: &str) -> ResultWrite {
    self.write(s.as_bytes()).map_err(|_| core::fmt::Error)?;
    Ok(())
  }

  fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> ResultWrite {
    if let Some(s) = args.as_str() {
      self.write_str(s)?;
      if s.len() > WRITE_BUFF_SIZE {
        self.write_str("[truncated] \n")?;
      };
    } else {
      let mut s: String<128> = String::new();
      match write!(&mut s, "{args}") {
        Ok(_) => self.write_str(&s)?,
        Err(_) => {
          self.write_str(&s)?;
          self.write_str("[truncated args] \n")?;
        }
      }
    }
    Ok(())
  }
}


// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Macros
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
            critical_section::with(|cs|  {
            $crate::serial_io::GLOBAL_SERIALIO
            .borrow_ref_mut(cs).as_mut()
            .map(|s| s.write_fmt(format_args!($($arg)*)))

    })}
}

#[macro_export]
macro_rules! println {
    () => {
        print!("\r\n")
    };
    ($($arg:tt)*) => {{
        print!($($arg)*);
        print!("\r\n");
    }};
}
