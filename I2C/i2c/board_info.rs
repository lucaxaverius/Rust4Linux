// board_info.rs

//! Module for I2C board information.
//!
//! This module provides the `I2CBoardInfo` struct, which contains information
//! about I2C devices present on the board.

use kernel::bindings;
use core::ffi::c_char;
use crate::i2c::utils::{make_device_name, I2C_NAME_SIZE};

/// Represents information about an I2C device on the board.
///
/// Used to declare I2C devices that are not auto-detected.
#[repr(transparent)]
pub struct I2CBoardInfo {
    /// The inner `i2c_board_info` struct from the kernel.
    inner: bindings::i2c_board_info,
}

const ZEROED_I2C_BOARD_INFO: bindings::i2c_board_info = bindings::i2c_board_info {
    type_: [0 as c_char; I2C_NAME_SIZE],
    flags: 0,
    addr: 0,
    dev_name: core::ptr::null_mut(),
    platform_data: core::ptr::null_mut(),
    of_node: core::ptr::null_mut(),
    fwnode: core::ptr::null_mut(),
    swnode: core::ptr::null_mut(),
    resources: core::ptr::null_mut(),
    num_resources: 0,
    irq: 0,
};

impl I2CBoardInfo {
    /// Creates a new `I2CBoardInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `dev_type` - The device type as a byte slice.
    /// * `dev_addr` - The I2C address of the device.
    ///
    /// # Example
    ///
    /// ```rust
    /// const BOARD_INFO: I2CBoardInfo = I2CBoardInfo::new(b"my_device", 0x50);
    /// ```
    pub const fn new(dev_type: &[u8], dev_addr: u16) -> Self {
        let type_array = make_device_name(dev_type);

        let inner = bindings::i2c_board_info {
            type_: type_array,
            addr: dev_addr,
            ..ZEROED_I2C_BOARD_INFO
        };

        I2CBoardInfo { inner }
    }

    /// Returns a copy of the inner `i2c_board_info` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned struct is used appropriately.
    pub const fn inner(&self) -> bindings::i2c_board_info {
        self.inner
    }

    /// Returns a pointer to the inner `i2c_board_info` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure the `I2CBoardInfo` remains valid while the pointer is in use.
    pub fn as_ptr(&self) -> *const bindings::i2c_board_info {
        &self.inner as *const bindings::i2c_board_info
    }
}

unsafe impl Sync for I2CBoardInfo {}
unsafe impl Send for I2CBoardInfo {}
