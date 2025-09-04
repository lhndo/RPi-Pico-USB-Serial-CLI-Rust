//! This module owns the serial interface and the usb device
// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                           Serial IO
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::cell::RefCell;
use core::fmt;
use core::fmt::Write;

use crate::delay::DELAY;

use critical_section::{Mutex, with as free};
use rp_pico::hal::usb::UsbBus;
use usb_device::UsbError;
use usb_device::device::UsbDevice;
use usbd_serial::SerialPort;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Globals
// ————————————————————————————————————————————————————————————————————————————————————————————————

// Used with poll_for_break_cmd()
const INTERRUPT_CHAR: u8 = b'~'; // char "~"

pub static SERIAL: SerialHandle = SerialHandle;
pub static SERIAL_CELL: Mutex<RefCell<Option<Serialio>>> = Mutex::new(RefCell::new(None));

pub type SerialDev = SerialPort<'static, UsbBus>;
pub type UsbDev = UsbDevice<'static, UsbBus>;
pub type Result<T> = core::result::Result<T, UsbError>;

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                              Init
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Initialise the SERIAL global object once
pub fn init(serial: SerialDev, usb_dev: UsbDev) {
  free(|cs| {
    let mut cell = SERIAL_CELL.borrow_ref_mut(cs);

    if cell.is_some() {
      panic!("SERIAL already initialized");
    }

    cell.replace(Serialio::new(serial, usb_dev));
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
  pub fn with<F, R>(&self, f: F) -> R
  where F: FnOnce(&mut Serialio) -> R {
    free(|cs| {
      if let Some(cell) = SERIAL_CELL.borrow_ref_mut(cs).as_mut() {
        f(cell)
      } else {
        panic!("SERIAL not initialized");
      }
    })
  }

  /// Polls the USB device and returns true if data was exchanged.
  pub fn poll_usb(&self) -> bool {
    self.with(|cell| cell.poll_usb())
  }

  /// Reads a line from the USB serial into the provided buffer.
  pub fn read_line_blocking(&self, buffer: &mut [u8]) -> Result<usize> {
    self.with(|cell| cell.read_line_blocking(buffer))
  }

  /// Writes data to the USB serial.
  pub fn write(&self, data: &[u8]) -> Result<()> {
    self.with(|cell| cell.write(data))
  }

  /// Get serial monitor connection flag
  pub fn is_connected(&self) -> bool {
    self.with(|cell| cell.serial.dtr())
  }

  /// flush the rx buffer discarding the data
  pub fn flush_rx(&self) {
    self.with(|cell| cell.flush_rx())
  }

  /// Polls for interrupt cmd though the serial read buffer
  /// This should be only called by the USB Interrupt
  pub fn poll_for_interrupt_char(&self) {
    self.with(|cell| cell.poll_for_interrupt())
  }

  /// Checks if an interrupt command was received via the USB serial.
  pub fn interrupt_cmd_triggered(&self) -> bool {
    self.with(|cell| cell.interrupt_cmd_triggered)
  }

  /// Clear the interrupt comand trigger state
  pub fn clear_interrupt_cmd(&self) {
    self.with(|cell| cell.interrupt_cmd_triggered = false);
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                         Serialio Struct
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub struct Serialio {
  pub serial: SerialDev,
  pub usb_dev: UsbDev,
  pub request_poll_for_interrupt: bool,
  pub interrupt_cmd_triggered: bool,
}

impl Serialio {
  fn new(serial: SerialDev, usb_dev: UsbDev) -> Self {
    Self {
      serial,
      usb_dev,
      request_poll_for_interrupt: false,
      interrupt_cmd_triggered: false,
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

  /// flush the rx buffer discarding the data
  fn flush_rx(&mut self) {
    let mut discard_buffer = [0u8; 64];

    // Keep reading until buffer is empty
    loop {
      match self.serial.read(&mut discard_buffer) {
        Ok(bytes_read) if bytes_read > 0 => {}
        _ => {
          // No more data or error, we're done
          break;
        }
      }
    }
  }

  /// Polls serial read buffer for an excape character (INTERRUPT_CHAR '~' )
  /// To be used in loops that need to be interrupted from the command line
  /// WARNING: This will throw away the read buffer
  fn poll_for_interrupt(&mut self) {
    // If no serial connection return false
    if !self.serial.dtr() {
      self.interrupt_cmd_triggered = false;
      return;
    }

    let mut buffer = [0u8; 64];

    loop {
      match self.serial.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {
          // Scan the buffer for interrupt character
          if buffer[..bytes_read].contains(&INTERRUPT_CHAR) {
            // Found interrupt character, flush and set flag
            self.interrupt_cmd_triggered = true;
            self.flush_rx();
            return;
          }
          continue;
        }
        _ => {
          // No data available
          self.interrupt_cmd_triggered = false;
          return;
        }
      }
    }
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
          // If not connected to serial, we exit
          if !self.serial.dtr() {
            return Err(UsbError::WouldBlock);
          }
          // Otherwise The serial buffer is full and we must keep polling
          // Small delay to avoid tight loops
          DELAY.us(6);
        }
        Err(e) => {
          // A different, real error occurred. We exit.
          return Err(e);
        }
      }

      // We must poll the USB device to send the serial data
      self.usb_dev.poll(&mut [&mut self.serial]);
    }

    Ok(())
  }

  /// Blocking read from serial into the provided buffer until a newline `\n`  is found.
  /// The newline character is not included in the buffer.
  ///
  /// If the line is longer than the buffer, the buffer is filled, the rest of the
  /// line is discarded from the serial input, and `Err(UsbError::BufferOverflow)` is returned.
  ///
  /// Returns the number of bytes written to the buffer on success.
  pub fn read_line_blocking(&mut self, buffer: &mut [u8]) -> Result<usize> {
    // No serial connection established, exit immediately.
    if !self.serial.dtr() {
      return Err(UsbError::InvalidEndpoint);
    }

    let mut bytes_read = 0;
    let buffer_len = buffer.len();
    let mut overflow = false;

    loop {
      //
      // Inner loop to read a single byte
      let byte = loop {
        self.poll_usb();

        let mut byte_buffer = [0u8; 1];
        match self.serial.read(&mut byte_buffer) {
          Ok(1) => break byte_buffer[0], // Got a byte, break inner loop.
          Ok(_) => {}                    // Read 0 bytes, should never happen...
          Err(UsbError::WouldBlock) => {
            // No data available, check connection and continue polling.
            if !self.serial.dtr() {
              // No serial connection, we exit.
              return Err(UsbError::InvalidEndpoint);
            }
            // Add a small delay to avoid a tight loop
            DELAY.us(6);
          }
          Err(e) => return Err(e), // Non-recoverable error occurred.
        }
      };

      // Check the byte for newline characters.
      if byte == b'\n' {
        if overflow {
          // We finished reading the oversized line. Return the error.
          return Err(UsbError::BufferOverflow);
        } else {
          // Done! End of line found and it fit in the buffer.
          return Ok(bytes_read);
        }
      }

      // It's a regular character.
      if bytes_read < buffer_len {
        // There is space, store the byte.
        buffer[bytes_read] = byte;
        bytes_read += 1;
      } else {
        // No more space, set overflow flag. We will now discard bytes.
        overflow = true;
      }
    }
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
        critical_section::with(|cs| {
            if let Some(s) = $crate::serial_io::SERIAL_CELL.borrow_ref_mut(cs).as_mut() {
                let _ = s.write_fmt(format_args!($($arg)*));
            }
        })
    }
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\r\n")
    };
    ($($arg:tt)*) => {
        critical_section::with(|cs| {
            if let Some(s) = $crate::serial_io::SERIAL_CELL.borrow_ref_mut(cs).as_mut() {
                let _ = writeln!(s, $($arg)*);
            }
        })
    };
}
