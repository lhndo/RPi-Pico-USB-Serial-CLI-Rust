// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          FIFO Buffer
// ————————————————————————————————————————————————————————————————————————————————————————————————
// Adapted From Anchor

/// Simple FIFO buffer implementation which can be useful when managing data to/from.
pub struct FifoBuffer<const BUF_SIZE: usize> {
  buffer:   [u8; BUF_SIZE],
  pub used: usize,
}

impl Default for FifoBuffer<128> {
  fn default() -> Self {
    FifoBuffer {
      buffer: [0u8; 128],
      used:   0,
    }
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Methods
// ————————————————————————————————————————————————————————————————————————————————————————————————

impl<const BUF_SIZE: usize> FifoBuffer<BUF_SIZE> {
  /// Creates a new buffer
  ///
  /// This is declared const, allowing it to be used even in `static const` contexts.
  pub const fn new() -> Self {
    FifoBuffer {
      buffer: [0u8; BUF_SIZE],
      used:   0,
    }
  }

  #[inline(always)]
  pub fn is_empty(&self) -> bool {
    self.used == 0
  }

  #[inline(always)]
  pub fn is_full(&self) -> bool {
    self.used == self.buffer.len()
  }

  /// Return length of currently stored buffer
  #[inline(always)]
  pub fn len(&self) -> usize {
    self.used
  }

  /// Get available size
  #[inline(always)]
  pub fn available(&self) -> usize {
    self.buffer.len() - self.used
  }

  /// Return mutable slice to the non-filled part of the buffer
  /// To be used with advance()
  #[inline(always)]
  pub fn receive_buffer(&mut self) -> &mut [u8] {
    &mut self.buffer[self.used..]
  }

  /// Add a single byte, return flase if full
  #[inline(always)]
  pub fn add_byte(&mut self, byte: u8) -> bool {
    if self.used + 1 >= self.buffer.len() {
      return false;
    }

    self.buffer[self.used + 1] = byte;
    self.used += 1;
    true
  }

  /// Append / Write `buf` to the non-filled part of the buffer
  /// Return none if buffer is full
  /// Return usize written
  #[inline(always)]
  pub fn append(&mut self, buf: &[u8]) -> Option<usize> {
    let into = self.receive_buffer();
    let len = into.len().min(buf.len());

    if len == 0 {
      return None;
    }

    into[..len].copy_from_slice(&buf[..len]);
    self.used = (self.used + len).clamp(0, self.buffer.len());
    Some(len)
  }

  /// Moves the used cursor forward
  ///
  /// This can be used after filling part of the non-filled buffer returned by `receive_buffer`.
  #[inline(always)]
  pub fn advance(&mut self, n: usize) {
    self.used = (self.used + n).clamp(0, self.buffer.len());
  }

  /// Returns the filled part of the buffer
  #[inline(always)]
  pub fn data(&self) -> &[u8] {
    &self.buffer[0..self.used]
  }

  /// Moves the data into a provided slice. Pops the read count. Returns transfered size.
  #[inline(always)]
  pub fn read(&mut self, data: &mut [u8]) -> usize {
    let len = self.used.min(data.len());
    if len == 0 {
      return 0;
    }
    data[..len].copy_from_slice(&self.buffer[..len]);
    self.pop(len);
    len
  }

  /// Read and pop the first byte
  #[inline(always)]
  pub fn read_byte(&mut self, data: &mut [u8]) -> Option<u8> {
    if self.used == 0 {
      return None;
    }

    let byte = self.buffer[0];
    self.pop(1);
    Some(byte)
  }

  /// Removes `n` bytes from the front of the buffer
  ///
  /// This operation moves the used part of the buffer down in memory. This is linear in the
  /// number of bytes currently stored.
  #[inline(always)]
  pub fn pop(&mut self, n: usize) {
    let n = n.clamp(0, self.used);
    let remain = n..self.used;
    let len = remain.len();
    self.buffer.copy_within(remain, 0);
    self.used = len;
  }

  /// Returns first byte index found or None
  #[inline(always)]
  pub fn contains_byte(&self, byte: u8) -> Option<usize> {
    for (index, &b) in self.data().iter().take(self.used).enumerate() {
      if b == byte {
        return Some(index);
      }
    }
    None
  }

  /// Searches for a string slice and returns Some(usize) if found, None if not
  #[inline(always)]
  pub fn contains_str(&self, word: &str) -> Option<usize> {
    let pattern = word.as_bytes();
    self.contains_slice(pattern)
  }

  /// Searches for a string slice and returns Some(usize) if found, None if not
  #[inline(always)]
  pub fn contains_slice(&self, slice: &[u8]) -> Option<usize> {
    let buffer = self.data();

    if buffer.len() < slice.len() {
      return None;
    }
    for (index, window) in buffer.windows(slice.len()).enumerate() {
      if window == slice {
        return Some(index);
      }
    }
    None
  }

  /// Sets buffer end point at index
  #[inline(always)]
  pub fn set_end(&mut self, index: usize) {
    self.used = index.clamp(0, self.buffer.len());
  }

  /// Clears the buffer
  #[inline(always)]
  pub fn clear(&mut self) {
    self.used = 0;
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

pub trait AsStr {
  fn as_str(&self) -> &str;
}

impl AsStr for [u8] {
  /// Tries to convert an u8 array to utf8 &str. Defaults to error str if it fails.
  fn as_str(&self) -> &str {
    core::str::from_utf8(self).unwrap_or("Err: utf8 conversion")
  }
}
