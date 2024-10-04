// SPDX-License-Identifier: GPL-2.0

//! I2C support.
//!
//! This module contains the kernel APIs related to I2C that have b>
//! wrapped for usage by Rust code in the kernel.

use kernel::prelude::*;
use kernel::bindings;

/// Represents an I2C adapter in the Rust kernel.
/// This structure wraps the C `i2c_adapter` struct.
#[repr(C)]
pub struct I2CAdapter {
    // Pointer to the C version of the adapter structure
    ptr: *mut bindings::i2c_adapter,
}

impl I2CAdapter {
    /// Creates a new `I2CAdapter` instance with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `owner` - The owner module (`THIS_MODULE`).
    /// * `algo` - The I2C algorithm for the adapter.
    /// * `class` - The I2C class.
    ///
    /// This function initializes the `i2c_adapter` structure and returns a new `I2CAdapter` instance.
    pub fn new(owner: *mut bindings::module, algo: *const bindings::i2c_algorithm, class: u32) -> Self {
        let adapter = bindings::i2c_adapter {
            owner,
            algo,
            class,
            ..Default::default()
        };

        let adapter_ptr = Box::into_raw(Box::new(adapter,GFP_KERNEL).expect("Failed during adapter creation"));

        I2CAdapter { ptr: adapter_ptr }
    }


    /// Registers the I2C adapter with the kernel.
    ///
    /// This function uses the `i2c_add_adapter` function from the C kernel API to
    /// register the adapter with the I2C subsystem.
    ///
    /// # Safety
    ///
    /// This function performs an FFI call and should be used with caution.
    /// Ensure that the adapter pointer is valid.
    pub unsafe fn add_adapter(&self) -> Result {
        let res = unsafe{bindings::i2c_add_adapter(self.ptr)};
            if res != 0 {
                return Err(EINVAL);
            }
            Ok(())
    }

    /// Removes the I2C adapter from the kernel.
    ///
    /// This function uses `i2c_del_adapter` to unregister the adapter from the I2C subsystem.
    pub unsafe fn del_adapter(&self) {
        unsafe{ 
            bindings::i2c_del_adapter(self.ptr);
        }
    }
}

/// Represents an I2C client device.
///
/// This structure wraps the C `i2c_client` struct and provides methods for communication
/// with I2C devices.
#[repr(C)]
pub struct I2CClient {
    ptr: *mut bindings::i2c_client,
}

impl I2CClient {
    /// Creates a new `I2CClient` instance with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `adapter` - The adapter to which the client is attached.
    /// * `address` - The I2C address of the device.
    ///
    /// This function initializes the `i2c_client` structure and returns a new `I2CClient` instance.
    pub fn new(adapter: *mut bindings::i2c_adapter, address: u16) -> Self {
        let client = bindings::i2c_client {
            addr: address,
            adapter,
            ..Default::default()
        };

        let client_ptr = Box::into_raw(Box::new(client,GFP_KERNEL).expect("Failed during client creation"));

        I2CClient { ptr: client_ptr }
    }

    /// Reads a byte from the specified I2C device register.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register address to read from.
    ///
    /// # Safety
    ///
    /// This function performs an unsafe FFI call to `i2c_smbus_read_byte_data` and should be
    /// used with valid pointers and register addresses.
    pub unsafe fn read_byte(&self, reg: u8) -> Result<u8> {
        let result = unsafe{bindings::i2c_smbus_read_byte_data(self.ptr, reg)};
            if result < 0 {
                return Err(EINVAL);
            }
            Ok(result as u8)
        
    }

    /// Writes a byte to the specified I2C device register.
    ///
    /// # Arguments
    ///
    /// * `reg` - The register address to write to.
    /// * `value` - The value to write to the register.
    ///
    /// # Safety
    ///
    /// This function performs an unsafe FFI call to `i2c_smbus_write_byte_data` and should be
    /// used with valid pointers and register addresses.
    pub unsafe fn write_byte(&self, reg: u8, value: u8) -> Result {
        let res = unsafe{bindings::i2c_smbus_write_byte_data(self.ptr, reg, value as u8)};
        if res < 0 {
            return Err(EINVAL);
        }
        Ok(())

    }
}

/// Represents an I2C driver.
///
/// This structure wraps the C `i2c_driver` struct and provides methods for driver registration
/// and deregistration with the I2C subsystem.
pub struct I2CDriver {
    ptr: *mut bindings::i2c_driver,
}

impl I2CDriver {
    /// Creates a new `I2CDriver` instance with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the driver.
    /// * `probe` - The probe function for the driver.
    /// * `remove` - The remove function for the driver.
    /// * `module` - The owner module (`THIS_MODULE`).
    /// * `id_table` - The device ID table for matching devices.
    /// * `address_list` - A list of I2C addresses to probe.

    ///
    /// This function initializes the `i2c_driver` structure and returns a new `I2CDriver` instance.
    pub fn new(
        name: *const i8,
        probe: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32>,
        remove: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client)>,
        module: *mut bindings::module,
        id_table: *const bindings::i2c_device_id,  
        address_list: *const u16,  
    ) -> Self {
        let driver = bindings::i2c_driver {
            driver: bindings::device_driver {
                name,
                owner: module,
                ..Default::default()
            },
            probe,
            remove,
            id_table,
            address_list,
            ..Default::default()
        };

        let driver_ptr = Box::into_raw(Box::new(driver,GFP_KERNEL).expect("Failed during driver creation"));

        I2CDriver { ptr: driver_ptr }
    }

    /// Registers the I2C driver with the kernel.
    ///
    /// # Arguments
    ///
    /// * `module_ptr` - A pointer to the current kernel module (`THIS_MODULE`).
    ///
    /// # Safety
    ///
    /// The caller must ensure that the passed module pointer is valid and corresponds to `THIS_MODULE`.
    pub unsafe fn register_driver(&self, module_ptr: *mut bindings::module) -> Result {
        
        let res = unsafe{bindings::i2c_register_driver(module_ptr, self.ptr)};       
        if res != 0 {
            return Err(EINVAL);
        }
        Ok(())
        
    }

    /// Deregisters the I2C driver from the kernel.
    ///
    /// This function calls `i2c_del_driver` to remove the driver from the I2C subsystem.
    pub unsafe fn remove_driver(&self) {
        unsafe{bindings::i2c_del_driver(self.ptr);}
    }
}

