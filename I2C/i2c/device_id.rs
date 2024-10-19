// device_id.rs

//! Module for I2C device identification.
//!
//! This module provides the `I2CDeviceID` struct, representing device identifiers
//! used for driver matching.

use kernel::bindings;
use crate::i2c::utils::{make_device_name, I2C_NAME_SIZE};

/// Represents an I2C device ID used for driver matching.
///
/// This struct wraps the kernel's `i2c_device_id` struct.
#[repr(transparent)]
pub struct I2CDeviceID {
    /// The inner `i2c_device_id` struct.
    pub inner: bindings::i2c_device_id,
}

impl I2CDeviceID {
    /// Creates a new `I2CDeviceID` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The device name as a byte slice.
    /// * `driver_data` - Driver-specific data.
    ///
    /// # Example
    ///
    /// ```rust
    /// static ID_TABLE: [I2CDeviceID; 2] = [
    ///     I2CDeviceID::new(b"my_device", 0),
    ///     I2CDeviceID::new(b"", 0), // Terminating entry
    /// ];
    /// ```
    pub const fn new(name: &[u8], driver_data: u64) -> Self {
        let name_array = make_device_name(name);
        Self {
            inner: bindings::i2c_device_id {
                name: name_array,
                driver_data,
            },
        }
    }

    /// Converts an array of `I2CDeviceID` to an array of `bindings::i2c_device_id`.
    ///
    /// # Arguments
    ///
    /// * `array` - A reference to an array of `I2CDeviceID`.
    ///
    /// # Returns
    ///
    /// An array of `bindings::i2c_device_id` suitable for kernel consumption.
    pub const fn to_bindings_array<const N: usize>(
        array: &[I2CDeviceID; N],
    ) -> [bindings::i2c_device_id; N] {
        let mut result: [bindings::i2c_device_id; N] = [bindings::i2c_device_id {
            name: [0; I2C_NAME_SIZE],
            driver_data: 0,
        }; N];
        let mut i = 0;
        while i < N {
            result[i] = array[i].inner;
            i += 1;
        }
        result
    }
}

unsafe impl Sync for I2CDeviceID {}
