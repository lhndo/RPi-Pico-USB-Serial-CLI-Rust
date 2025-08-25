//! This module owns the serial interface, the method of communicating outwards
//! such as the usb device, and a write buffer
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Serial IO
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::cell::RefCell;
use core::fmt;
use core::fmt::Write;

use crate::delay::DELAY;

use rp_pico::hal::usb::UsbBus;

use cortex_m::interrupt::{Mutex, free};
use usb_device::UsbError;
use usb_device::device::UsbDevice;
use usbd_serial::SerialPort;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub static SERIAL: SerialHandle = SerialHandle;

pub static SERIAL_CELL: Mutex<RefCell<Option<Serialio>>> = Mutex::new(RefCell::new(None));

// Used with poll_for_break_cmd()
const INTERRUPT_CHAR: u8 = 0x7e; // char "~"

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Init
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Initialise the SERIAL global object once
pub fn init(serial: SerialDev, usb_dev: UsbDev) {
  free(|cs| {
    let mut cell = SERIAL_CELL.borrow(cs).borrow_mut();

    if cell.is_some() {
      panic!("SERIAL already initialized");
    }

    let serialio = Serialio::new(serial, usb_dev);
    *cell = Some(serialio);
  });
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                      SerialHandle Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Serial Handle for the GLOBAL SERIAL object
pub struct SerialHandle;

// ————————————————————————————————————— SerialHandle impl ————————————————————————————————————————

impl SerialHandle {
  /// Executes a closure with a mutable reference to the serial peripheral.
  pub fn with_serial<F, R>(&self, f: F) -> R
  where F: FnOnce(&mut Serialio) -> R {
    free(|cs| {
      if let Some(serial) = SERIAL_CELL.borrow(cs).borrow_mut().as_mut() {
        f(serial)
      } else {
        panic!("SERIAL not initialized");
      }
    })
  }

  /// Polls the USB device and returns true if data was exchanged.
  pub fn poll_usb(&self) -> bool {
    self.with_serial(|s| s.poll_usb())
  }

  /// Checks if an interrupt command was received via the USB serial.
  pub fn poll_for_break_cmd(&self) -> bool {
    self.with_serial(|s| s.poll_for_break_cmd())
  }

  /// Reads a line from the USB serial into the provided buffer.
  pub fn read_line(&self, buffer: &mut [u8]) -> Result<usize> {
    self.with_serial(|s| s.read_line(buffer))
  }

  /// Writes data to the USB serial.
  pub fn write(&self, data: &[u8]) -> Result<()> {
    self.with_serial(|s| s.write(data))
  }

  /// Get drt status when serial monitor connection established
  pub fn get_drt(&self) -> bool {
    self.with_serial(|s| s.serial.dtr())
  }

  /// Set serial monitor connection flag
  pub fn set_connected(&self, value: bool) {
    self.with_serial(|s| s.connected = value);
  }

  pub fn update_connected_status(&self) {
    self.with_serial(|s| s.connected = s.serial.dtr());
  }

  pub fn is_connected(&self) -> bool {
    self.with_serial(|s| s.connected)
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
  pub serial:    SerialDev,
  pub usb_dev:   UsbDev,
  pub connected: bool,
}

impl Serialio {
  fn new(serial: SerialDev, usb_dev: UsbDev) -> Self {
    Self {
      serial,
      usb_dev,
      connected: false,
    }
  }

  // ——————————————————————————————————————————————————————————————————————————————————————————————
  //                                           Methods
  // ——————————————————————————————————————————————————————————————————————————————————————————————

  /// Polls the usb device for rx tx data, and returns true if some data was exchanged
  /// Must poll the usb for every 10ms to be compliant
  fn poll_usb(&mut self) -> bool {
    self.usb_dev.poll(&mut [&mut self.serial])
  }

  /// Polls serial read buffer for an excape character (INTERRUPT_CHAR '~' )
  /// Runs once, non interrupting. Returns true if found.
  /// To be used in loops that need to be interrupted from the command line
  /// WARNING: This will throw away the buffer
  fn poll_for_break_cmd(&mut self) -> bool {
    let mut found = false;
    if self.usb_dev.poll(&mut [&mut self.serial]) {
      let mut buffer = [0u8; 64];

      if let Ok(count) = self.serial.read(&mut buffer) {
        let received_data = &buffer[..count];
        if received_data.contains(&INTERRUPT_CHAR) {
          found = true;
        }
      }
    }
    found
  }

  /// Appends as much as possible into the write buffer
  /// Writes an entire slice of data, blocking until it is all sent.
  /// This function writes directly to the USB serial port in a loop.
  fn write(&mut self, mut data: &[u8]) -> Result<()> {
    while !data.is_empty() {
      // Try to write the remaining data to the serial port's internal buffer.
      match self.serial.write(data) {
        Ok(written) => {
          // If we wrote some bytes, advance the slice.
          data = &data[written..];
        }
        Err(UsbError::WouldBlock) => {
          // if serial monitor not we exit
          if !self.serial.dtr() {
            return Err(UsbError::WouldBlock);
          }

          // Otherwise The USB host isn't ready for data. This is normal.
          // We just need to wait and let the USB bus handle its events.
        }
        Err(e) => {
          // A different, real error occurred.
          return Err(e);
        }
      }

      // We must always poll the USB device in the loop. This allows
      // the device to respond to the host and manage the connection.
      self.usb_dev.poll(&mut [&mut self.serial]);
    }

    Ok(())
  }

  /// Blocking read from serial into provided buffer until new line. Discards the rest.
  /// Returns size written
  /// Remember to clear the read buffer beforehand
  pub fn read_line(&mut self, buffer: &mut [u8]) -> Result<usize> {
    let max_len = buffer.len();
    let mut used = 0;

    loop {
      // Try grabbing a new usb package only if serial is established
      while !self.poll_usb() && self.serial.dtr() {
        DELAY.us(5);
      }

      // Read as much as possible into the read buffer
      match self.serial.read(&mut buffer[used..]) {
        Ok(count) if count > 0 => {
          if let Some(index) = buffer[used..(used + count)].iter().position(|&b| b == b'\n') {
            used += index;
            break;
          }
          used += count;

          // if buffer is full but no new like, we error
          if used >= max_len {
            return Err(UsbError::BufferOverflow);
          }
        }
        Ok(_) => {
          // no data received, keep trying
          continue;
        }
        Err(usb_device::UsbError::WouldBlock) => {
          // if we dropped serial we exit
          if !self.serial.dtr() {
            return Err(UsbError::WouldBlock);
          } else {
            continue;
          }
        }
        Err(e) => {
          return Err(e);
        }
      }
    }

    self.flush_read_all()?; // Discarding the rest of the data since we got our line
    Ok(used)
  }

  /// Tries to flush the rest of the serial read into void. Not guaranteed
  /// it succeeds since we don't know if the host stopped sending messages
  fn flush_read_all(&mut self) -> Result<()> {
    const MAX_EMPTY_READ_ATTEMPTS: i32 = 20;
    let mut temp_buffer = [0u8; 64];
    let mut retries = MAX_EMPTY_READ_ATTEMPTS; // Limit retries to avoid infinite loop

    while retries > 0 {
      self.poll_usb();
      DELAY.us(5);

      match self.serial.read(&mut temp_buffer[..]) {
        Ok(count) if count > 0 => {
          retries = MAX_EMPTY_READ_ATTEMPTS; // Got data, reset retry count
          continue;
        }
        Ok(_) => {
          retries -= 1; // No data read, decrement retries
          continue;
        }
        Err(UsbError::WouldBlock) => {
          if !self.serial.dtr() {
            return Err(UsbError::WouldBlock);
          } else {
            // No data available right now
            retries -= 1;
            continue;
          }
        }
        Err(e) => {
          return Err(e);
        }
      }
    }

    Ok(())
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

// ——————————————————————————————————————————— Write ——————————————————————————————————————————————

impl Write for Serialio {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write(s.as_bytes()).map_err(|_| fmt::Error)?;
    Ok(())
  }

  fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
    core::fmt::write(self, args)?;
    Ok(())
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Macros
// ————————————————————————————————————————————————————————————————————————————————————————————————

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        ::cortex_m::interrupt::free(|cs| {
            if let Some(s) = $crate::serial_io::SERIAL_CELL.borrow(cs).borrow_mut().as_mut() {
                let _ = s.write_fmt(format_args!($($arg)*));
            }
        })
    }
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
