//! Quick way to bypass defmt logging for decreasing memory size for release builds

#[defmt::global_logger]
struct Logger;

unsafe impl defmt::Logger for Logger {
  fn acquire() {}

  unsafe fn flush() {}

  unsafe fn release() {}

  unsafe fn write(_bytes: &[u8]) {}
}
