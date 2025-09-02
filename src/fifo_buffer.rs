// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                          FIFO Buffer
// ————————————————————————————————————————————————————————————————————————————————————————————————

/// Simple generic FIFO buffer implementation.
pub struct FifoBuffer<const BUF_SIZE: usize> {
  buffer: [u8; BUF_SIZE],
  used:   usize,
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                            Methods
// ————————————————————————————————————————————————————————————————————————————————————————————————

// General methods for any type `T` that can be copied and has a default value.
impl<const BUF_SIZE: usize> FifoBuffer<BUF_SIZE> {
  /// Creates a new, empty buffer.
  pub fn new() -> Self {
    Self {
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
    self.used == BUF_SIZE
  }

  /// Returns the number of items in the buffer.
  #[inline(always)]
  pub fn len(&self) -> usize {
    self.used
  }

  /// Returns how many more items can be added.
  #[inline(always)]
  pub fn available(&self) -> usize {
    BUF_SIZE - self.used
  }

  /// Clears the buffer.
  #[inline(always)]
  pub fn clear(&mut self) {
    self.used = 0;
  }

  /// Moves the `used` cursor forward by `n` items.
  ///
  /// Useful after writing directly into the `receive_buffer`.
  #[inline(always)]
  pub fn advance(&mut self, n: usize) {
    self.used = self.used.saturating_add(n).min(BUF_SIZE);
  }

  /// Returns a mutable slice to the unused part of the buffer.
  /// Remember to set .advance(n) to set the endpoint
  #[inline(always)]
  pub fn receive_buffer(&mut self) -> &mut [u8] {
    &mut self.buffer[self.used..]
  }

  /// Adds a single item to the buffer. Returns `false` if full.
  #[inline(always)]
  pub fn add_single(&mut self, item: u8) -> bool {
    if self.is_full() {
      return false;
    }
    self.buffer[self.used] = item;
    self.used += 1;
    true
  }

  /// Appends items from a slice to the buffer.
  /// Returns the number of items written, or 0 if the buffer is full.
  #[inline(always)]
  pub fn append(&mut self, buf: &[u8]) -> usize {
    let into = self.receive_buffer();
    let len = into.len().min(buf.len());

    if len == 0 {
      return 0;
    }

    into[..len].copy_from_slice(&buf[..len]);
    self.advance(len);
    len
  }

  /// Returns a slice of the items currently in the buffer.
  #[inline(always)]
  pub fn data(&self) -> &[u8] {
    &self.buffer[0..self.used]
  }

  /// Reads items from the buffer into a provided slice.
  /// The read items are removed. Returns the number of items transferred.
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

  /// Reads and removes the first item from the buffer.
  #[inline(always)]
  pub fn read_single(&mut self) -> Option<u8> {
    if self.is_empty() {
      return None;
    }
    let item = self.buffer[0];
    self.pop(1);
    Some(item)
  }

  /// Removes `n` items from the front of the buffer.
  #[inline(always)]
  pub fn pop(&mut self, n: usize) {
    let n = n.min(self.used);
    self.buffer.copy_within(n..self.used, 0);
    self.used -= n;
  }

  /// Sets buffer's used length to a specific index.
  #[inline(always)]
  pub fn set_end(&mut self, index: usize) {
    self.used = index.min(BUF_SIZE);
  }
}

// Methods that require the type `T` to be comparable.
impl<const BUF_SIZE: usize> FifoBuffer<BUF_SIZE> {
  /// Returns the index of the first matching item, or `None`.
  #[inline(always)]
  pub fn contains(&self, item: &u8) -> Option<usize> {
    self.data().iter().position(|b| b == item)
  }

  /// Searches for a sub-slice and returns the starting index if found.
  #[inline(always)]
  pub fn contains_slice(&self, slice: &[u8]) -> Option<usize> {
    if slice.is_empty() {
      return None;
    };
    self.data().windows(slice.len()).position(|w| w == slice)
  }

  #[inline(always)]
  pub fn contains_str(&self, word: &str) -> Option<usize> {
    self.contains_slice(word.as_bytes())
  }
}

// ————————————————————————————————————————————————————————————————————————————————————————————————
//                                             Traits
// ————————————————————————————————————————————————————————————————————————————————————————————————

use core::str::Utf8Error;

// The AsStr trait can be defined and used separately for `u8` slices.
pub trait AsStr {
  fn as_str(&self) -> Result<&str, Utf8Error>;
}

impl AsStr for [u8] {
  /// Tries to convert a u8 slice to a utf8 &str.
  fn as_str(&self) -> Result<&str, Utf8Error> {
    core::str::from_utf8(self)
  }
}
