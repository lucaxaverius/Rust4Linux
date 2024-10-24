// msg.rs

//! Module for I2C message representation.
//!
//! This module defines the `I2CMsg` struct, which represents a single segment of an I2C transaction.


/// Represents a single segment of an I2C transaction.
///
/// An `I2CMsg` contains the address of the device, message flags, the length of the buffer,
/// and a pointer to the data buffer.
#[repr(C)]
pub struct I2CMsg {
    /// Slave address, either 7 or 10 bits.
    addr: u16,
    /// Message flags (e.g., read/write indicators).
    flags: u16,
    /// Length of the message buffer.
    len: u16,
    /// Pointer to the message buffer.
    buf: *mut u8,
}

impl I2CMsg {
    /// Creates a new `I2CMsg` instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - The slave address of the I2C device.
    /// * `flags` - Flags indicating the type of message (e.g., read/write).
    /// * `buf` - A mutable slice representing the data buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the buffer remains valid for the lifetime of the `I2CMsg`.
    pub fn new(addr: u16, flags: u16, buf: &mut [u8]) -> Self {
        I2CMsg {
            addr,
            flags,
            len: buf.len() as u16,
            buf: buf.as_mut_ptr(),
        }
    }

    // Constants for I2C message flags

    /// Flag indicating a read operation (from slave to master).
    pub const I2C_M_RD: u16 = 0x0001;
    /// Flag for 10-bit addressing.
    pub const I2C_M_TEN: u16 = 0x0010;
    /// Flag indicating the buffer is DMA safe.
    pub const I2C_M_DMA_SAFE: u16 = 0x0200;
    /// Flag indicating the message length will be the first received byte.
    pub const I2C_M_RECV_LEN: u16 = 0x0400;
    /// Flag to skip the ACK/NACK bit in read messages.
    pub const I2C_M_NO_RD_ACK: u16 = 0x0800;
    /// Flag to treat NACK from client as ACK.
    pub const I2C_M_IGNORE_NAK: u16 = 0x1000;
    /// Flag to reverse the direction of the transfer.
    pub const I2C_M_REV_DIR_ADDR: u16 = 0x2000;
    /// Flag to skip the repeated start condition.
    pub const I2C_M_NOSTART: u16 = 0x4000;
    /// Flag to force a stop condition after the message.
    pub const I2C_M_STOP: u16 = 0x8000;

    /// Returns a reference to the buffer as a slice for reading.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `buf` is a valid pointer and `len` is accurate.
    pub fn read_from_buf(&self) -> &[u8] {
        unsafe { alloc::slice::from_raw_parts(self.buf, self.len as usize) }
    }

    /// Returns a mutable reference to the buffer as a slice for writing.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `buf` is a valid pointer and `len` is accurate.
    pub fn write_to_buf(&mut self) -> &mut [u8] {
        unsafe { alloc::slice::from_raw_parts_mut(self.buf, self.len as usize) }
    }
}
