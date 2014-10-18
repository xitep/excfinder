#![allow(dead_code)]

// ~ a workaround for #16167

use std::io::IoResult;

#[deriving(Clone)]
pub struct MemWriter {
    buf: Vec<u8>,
}

impl MemWriter {
    /// Create a new `MemWriter`.
    #[inline]
    pub fn new() -> MemWriter {
        MemWriter::with_capacity(512)
    }
    /// Create a new `MemWriter`, allocating at least `n` bytes for
    /// the internal buffer.
    #[inline]
    pub fn with_capacity(n: uint) -> MemWriter {
        MemWriter { buf: Vec::with_capacity(n) }
    }


    /// Acquires an immutable reference to the underlying buffer of this
    /// `MemWriter`.
    #[inline]
    pub fn get_ref<'a>(&'a self) -> &'a [u8] { self.buf.as_slice() }

    /// Unwraps this `MemWriter`, returning the underlying buffer
    #[inline]
    pub fn unwrap(self) -> Vec<u8> { self.buf }


    // my addititions -------------------------------------------------

    #[inline]
    pub fn wrap(buf: Vec<u8>) -> MemWriter {
        MemWriter { buf: buf }
    }

    #[inline]
    pub fn clear(&mut self) { unsafe { self.buf.set_len(0); } }

    #[inline]
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
}

impl Writer for MemWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.buf.push_all(buf);
        Ok(())
    }
}
