// adapter.rs

//! Module for I2C adapter representation.
//!
//! This module provides the `I2CAdapter` struct, representing an I2C bus adapter,
//! and methods for interacting with the I2C bus.

use kernel::prelude::*;
use kernel::bindings;
use crate::i2c::msg::I2CMsg;
use crate::error::to_result;

/// Represents an I2C adapter (bus).
///
/// An `I2CAdapter` identifies a physical I2C bus and provides methods to perform
/// I2C transactions on that bus.
pub struct I2CAdapter {
    /// Pointer to the underlying `i2c_adapter` struct.
    ptr: *mut bindings::i2c_adapter,
}

impl I2CAdapter {
    /// Attempts to obtain an `I2CAdapter` from a given bus number.
    ///
    /// # Arguments
    ///
    /// * `bus_number` - The bus number of the I2C adapter to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(I2CAdapter)` if successful.
    /// * `Err(Error)` if the adapter cannot be found.
    pub fn get_from_bus_number(bus_number: i32) -> Result<Self> {
        // Safety: Calling the C API `i2c_get_adapter` which returns a pointer to `i2c_adapter` or null.
        let adapter_ptr = unsafe { bindings::i2c_get_adapter(bus_number) };

        if adapter_ptr.is_null() {
            Err(EINVAL)
        } else {
            // Safety: The pointer is non-null and valid
            Ok(Self { ptr: adapter_ptr })
        }
    }

    /// Performs an I2C transfer on the bus.
    ///
    /// # Arguments
    ///
    /// * `msgs` - A mutable slice of `I2CMsg` structs representing the messages to transfer.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` indicating the number of messages transferred.
    /// * `Err(Error)` if the transfer fails.
    pub fn transfer(&self, msgs: &mut [I2CMsg]) -> Result<usize> {
        // I2C not yet initialized
        if self.ptr.is_null() {
            return Err(EINVAL); 
        }
        
        let ret = unsafe {
            bindings::i2c_transfer(
                self.ptr,
                msgs.as_mut_ptr() as *mut bindings::i2c_msg,
                msgs.len() as i32, //the number of messages
            )
        };
        to_result(ret).map(|_| ret as usize)
    }
    
    /// Returns a raw pointer to the underlying `i2c_adapter` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned pointer is used safely and that
    /// the `I2CAdapter` remains valid as long as the pointer is used.
    pub fn as_ptr(&self) -> *mut bindings::i2c_adapter {
        self.ptr
    }
}

impl Drop for I2CAdapter {
    fn drop(&mut self) {
        // Release the reference to the adapter.
        unsafe { bindings::i2c_put_adapter(self.ptr) };
    }
}
