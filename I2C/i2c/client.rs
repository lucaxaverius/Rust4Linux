// client.rs

//! Module for I2C client representation.
//!
//! This module provides the `I2CClient` struct, representing an I2C slave device,
//! and methods for communicating with the device.

use kernel::prelude::*;
use kernel::bindings;
use core::ffi::{c_char};
use crate::i2c::adapter::I2CAdapter;
use crate::i2c::board_info::I2CBoardInfo;
use crate::error::to_result;

/// Represents an I2C client device.
///
/// An `I2CClient` is used to communicate with a specific I2C slave device on the bus.
pub struct I2CClient {
    /// Pointer to the underlying `i2c_client` struct.
    pub ptr: *mut bindings::i2c_client,
    owned: bool,
}

impl I2CClient {
    /// Creates a new `I2CClient` instance.
    ///
    /// # Arguments
    ///
    /// * `adapter` - The `I2CAdapter` to which the client is connected.
    /// * `board_info` - Information about the I2C device.
    ///
    /// # Returns
    ///
    /// * `Ok(I2CClient)` if successful.
    /// * `Err(Error)` if creation fails.
    pub fn new_client_device(adapter: &I2CAdapter, board_info: &I2CBoardInfo) -> Result<Self> {
        let client_ptr =
            unsafe { bindings::i2c_new_client_device(adapter.ptr, board_info.as_ptr()) };

        if client_ptr.is_null() || (client_ptr as isize) < 0 {
            Err(EINVAL)
        } else {
            Ok(Self {
                ptr: client_ptr,
                owned: true,
            })
        }
    }

    /// Creates an `I2CClient` from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the pointer is valid.
    pub unsafe fn from_raw_ptr(ptr: *mut bindings::i2c_client) -> Self {
        Self { ptr, owned: false }
    }

    /// Sends data to the I2C client device.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing the data to send.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` indicating the number of bytes sent.
    /// * `Err(Error)` if the send operation fails.
    pub fn master_send(&self, buf: &[c_char]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_send(self.ptr, buf.as_ptr(), buf.len() as i32) };
        to_result(ret).map(|_| ret as usize)
    }

    /// Receives data from the I2C client device.
    ///
    /// # Arguments
    ///
    /// * `buf` - A mutable byte slice to store the received data.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` indicating the number of bytes received.
    /// * `Err(Error)` if the receive operation fails.
    pub fn master_recv(&self, buf: &mut [c_char]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret =
            unsafe { bindings::i2c_master_recv(self.ptr, buf.as_mut_ptr(), buf.len() as i32) };
        to_result(ret).map(|_| ret as usize)
    }

    /// This executes the SMBus "send byte" protocol. 
    /// Writes a single byte to the I2C client device without specifying a device register.
    /// Some devices are so simple that this interface is enough; 
    /// for others, it is a shorthand if you want to read the same register as in the previous SMBus command.
    /// 
    /// # Arguments
    /// * `value` - The byte value to be written.
    /// 
    /// # Returns
    /// * `Ok(())` if the byte is successfully written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn send_byte(&self, value: u8) -> Result<()> {
        let ret = unsafe { bindings::i2c_smbus_write_byte(self.ptr, value) };
        to_result(ret)
    }

    /// This executes the SMBus "receive byte" protocol.
    /// Reads a single byte from the I2C client device without specifying a device register. 
    /// Some devices are so simple that this interface is enough; 
    /// for others, it is a shorthand if you want to read the same register as in the previous SMBus command.
    ///
    /// # Returns
    /// * `Ok(u8)` if the byte is successfully read.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn receive_byte(&self) -> Result<u8> {
        let ret = unsafe { bindings::i2c_smbus_read_byte(self.ptr) };
        if ret < 0 {
            Err(Error::from_errno(ret))
        } else {
            Ok(ret as u8)
        }
    }

    /// This executes the SMBus "write byte" protocol with a command.
    /// Writes a byte to a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command to which the byte should be written.
    /// * `value` - The byte value to be written.
    ///
    /// # Returns
    /// * `Ok(())` if the byte is successfully written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn write_byte(&self, command: u8, value: u8) -> Result<()> {
        let ret = unsafe { bindings::i2c_smbus_write_byte_data(self.ptr, command, value) };
        to_result(ret)
    }

    /// This executes the SMBus "read byte" protocol with a command.
    /// Reads a byte from a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command from which the byte should be read.
    ///
    /// # Returns
    /// * `Ok(u8)` if the byte is successfully read.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn read_byte(&self, command: u8) -> Result<u8> {
        let ret = unsafe { bindings::i2c_smbus_read_byte_data(self.ptr, command) };
        if ret < 0 {
            Err(Error::from_errno(ret))
        } else {
            Ok(ret as u8)
        }
    }

    /// This executes the SMBus "write word" protocol with a command.
    /// Writes a word to a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command to which the word should be written.
    /// * `value` - The word value to be written.
    ///
    /// # Returns
    /// * `Ok(())` if the word is successfully written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn write_word(&self, command: u8, value: u16) -> Result<()> {
        let ret = unsafe { bindings::i2c_smbus_write_word_data(self.ptr, command, value) };
        to_result(ret)
    }

    /// This executes the SMBus "read word" protocol with a command.
    /// Reads a word from a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command from which the word should be read.
    ///
    /// # Returns
    /// * `Ok(u16)` if the word is successfully read.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn read_word(&self, command: u8) -> Result<u16> {
        let ret = unsafe { bindings::i2c_smbus_read_word_data(self.ptr, command) };
        if ret < 0 {
            Err(Error::from_errno(ret))
        } else {
            Ok(ret as u16)
        }
    }

    /// This executes the SMBus "block write" protocol with a command.
    /// Writes a block of data to a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command to which the block should be written.
    /// * `values` - The block of data to be written (maximum 32 bytes).
    ///
    /// # Returns
    /// * `Ok(())` if the block is successfully written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn write_block(&self, command: u8, values: &[u8]) -> Result<()> {
        if values.len() > 32 {
            return Err(EINVAL);
        }
        let ret = unsafe {
            bindings::i2c_smbus_write_block_data(
                self.ptr,
                command,
                values.len() as u8,
                values.as_ptr() as *const u8,
            )
        };
        to_result(ret)
    }

    /// This executes the SMBus "block read" protocol with a command.
    /// Reads a block of data from a specific register (command) of the I2C client device.
    ///
    /// # Arguments
    /// * `command` - The register/command from which the block should be read.
    /// * `buf` - A mutable buffer (`&mut [u8]`) to store the data read from the slave. Maximum block size is 32 bytes.
    ///
    /// # Returns
    /// * `Ok(usize)` if the block is successfully read, indicating the number of bytes read.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn read_block(&self, command: u8, buf: &mut [u8]) -> Result<usize> {
        // Ensure the buffer length does not exceed the maximum block size (32 bytes).
        if buf.len() > 32 {
            return Err(EINVAL);
        }

        let ret = unsafe {
            bindings::i2c_smbus_read_block_data(
                self.ptr,
                command,
                buf.as_mut_ptr(),
            )
        };

        if ret < 0 {
            Err(Error::from_errno(ret))
        } else {
            Ok(ret as usize)
        }
    }
    
}

impl Drop for I2CClient {
    fn drop(&mut self) {
        if self.owned {
            // Unregister the I2C client device.
            unsafe { bindings::i2c_unregister_device(self.ptr) };
        }
    }
}
