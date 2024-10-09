// SPDX-License-Identifier: GPL-2.0

//! I2C support.
//!
//! This module contains the kernel APIs related to I2C that have b>
//! wrapped for usage by Rust code in the kernel.

use kernel::prelude::*;
use kernel::bindings;
use kernel::error::{Error, Result};
use crate::{types::Opaque};

/// This structure wraps the C `i2c_adapter` struct.
/// It is used to identify a physical i2c bus along
/// with the access algorithms necessary to access it.
#[repr(transparent)]
pub struct I2CAdapter(Opaque<bindings::i2c_adapter>);


impl I2CAdapter {
    /// Attempts to get an I2C adapter from a given bus number.
    /// 
    /// # Arguments
    /// * `bus_number` - The bus number for which the I2C adapter is requested.
    /// 
    /// # Returns
    /// * `Ok(I2CAdapter)` if an adapter is found.
    /// * `Err(Error)` if the adapter cannot be found (i.e., null pointer returned).
    pub fn get_from_bus_number(bus_number: i32) -> Result<I2CAdapter> {
        // Safety: Calling the C API `i2c_get_adapter` which returns a pointer to `i2c_adapter` or null.
        let adapter_ptr = unsafe { bindings::i2c_get_adapter(bus_number) };

        if adapter_ptr.is_null() {
            Err(Error::EINVAL)
        } else {
            // Safety: The pointer is non-null, so it's safe to create an I2CAdapter instance.
            Ok(I2CAdapter(unsafe { Opaque::new(adapter_ptr) }))
        }
    }
}

/// This structure wraps the C `i2c_board_info` struct.
/// It is used to build tables of information listing I2C devices
/// that are present. This information is used to grow the driver model tree.
#[repr(transparent)]
pub struct I2CBoardInfo(Opaque<bindings::i2c_board_info>);

/// Macro to initialize an `I2CBoardInfo` structure.
/// 
/// # Arguments
/// * `dev_type` - The device type identifier.
/// * `dev_addr` - The device address on the bus.
/// 
/// # Returns
/// * `I2CBoardInfo` instance with the provided type and address.
#[macro_export]
macro_rules! I2C_BOARD_INFO {
    ($dev_type:expr, $dev_addr:expr) => {{
        I2CBoardInfo(unsafe {
            let mut info: bindings::i2c_board_info = core::mem::zeroed();
            info.type_ = $dev_type;
            info.addr = $dev_addr;
            Opaque::new(&mut info)
        })
    }};
}


/// This structure wraps the C `i2c_device_id` struct.
#[repr(transparent)]
pub struct I2CDeviceId(Opaque<bindings::i2c_device_id>);


/// Macro to create a MODULE_DEVICE_TABLE equivalent in Rust.
/// 
/// # Arguments
/// * `type_` - The type of the device table.
/// * `name` - The name of the device table.
#[macro_export]
macro_rules! MODULE_DEVICE_TABLE {
    ($type_:ident, $name:ident) => {
        #[no_mangle]
        pub static mut __mod_${type_}__${name}_device_table: *const $crate::I2CDeviceId = &$name;
    };
}


/// This structure wraps the C `i2c_client`, it represents an I2C slave device (i.e. chip)
/// connected to ani i2c bus. 
///
/// The behaviour exposed to Linux is defined by the driver managing the device.
#[repr(transparent)]
pub struct I2CClient(Opaque<bindings::i2c_client>);

impl I2CClient {
    /// Creates a new `I2CClient` instance by calling the C function `i2c_new_client_device`.
    /// 
    /// # Arguments
    /// * `adapter` - The I2C adapter to which the client is connected.
    /// * `board_info` - The board information describing the client device.
    /// 
    /// # Returns
    /// * `Ok(I2CClient)` if the client is successfully created.
    /// * `Err(Error)` if the client cannot be created (i.e., error pointer returned).
    pub fn new(adapter: &I2CAdapter, board_info: &I2CBoardInfo) -> Result<I2CClient> {
        // Safety: Calling the C API `i2c_new_client_device` which returns a pointer to `i2c_client` or an error pointer.
        let client_ptr = unsafe { bindings::i2c_new_client_device(adapter.0.as_ptr(), board_info.0.as_ptr()) };

        if client_ptr.is_null() || (client_ptr as isize) < 0 {
            Err(EINVAL)
        } else {
            // Safety: The pointer is non-null and valid, so it's safe to create an I2CClient instance.
            Ok(I2CClient(unsafe { Opaque::new(client_ptr) }))
        }
    }


    /// Sends a single message to the I2C client device.
    /// 
    /// # Arguments
    /// * `buf` - A buffer containing the data to be sent. Must be shorter than 2^16 since msg.len is u16.
    /// 
    /// # Returns
    /// * `Ok(usize)` if the data is successfully sent, indicating the number of bytes written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn master_send(&self, buf: &[u8]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(Error::EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_send(self.0.as_ptr(), buf.as_ptr() as *const i8, buf.len() as i32) };
        Error::to_result(ret).map(|_| ret as usize)
    }

    /// Writes a single byte to the I2C client device.
    /// 
    /// # Arguments
    /// * `value` - The byte value to be written.
    /// 
    /// # Returns
    /// * `Ok(())` if the byte is successfully written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn write_byte(&self, value: u8) -> Result<()> {
        let ret = unsafe { bindings::i2c_smbus_write_byte(self.0.as_ptr(), value as i32) };
        Error::to_result(ret)
    }
}



/// Represents an I2C driver.
///
/// This structure wraps the C `i2c_driver` struct and provides methods for driver registration
/// and deregistration with the I2C subsystem.
#[repr(transparent)]
pub struct I2CDriver(Opaque<bindings::i2c_driver>);

impl I2CDriver {
    /// Registers the I2C driver with the kernel.
    /// 
    /// # Arguments
    /// * `driver` - The I2C driver to be registered.
    /// 
    /// # Returns
    /// * `Ok(())` if the driver is successfully registered.
    /// * `Err(Error)` if registration fails.
    pub fn add_driver(driver: &I2CDriver) -> Result<()> {
        let ret = unsafe { bindings::i2c_add_driver(driver.0.as_ptr()) };
        Error::to_result(ret)
    }

    /// Unregisters the I2C driver from the kernel.
    /// 
    /// # Arguments
    /// * `driver` - The I2C driver to be unregistered.
    pub fn del_driver(driver: &I2CDriver) {
        unsafe { bindings::i2c_del_driver(driver.0.as_ptr()) };
    }
}

/// Implement Send and Sync for I2CDriver
unsafe impl Send for I2CDriver {}
unsafe impl Sync for I2CDriver {}


/// A builder for creating an `I2CDriver` instance.
pub struct I2CDriverBuilder {
    class: Option<u32>,
    probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
    remove: unsafe extern "C" fn(client: *mut bindings::i2c_client)>,
    shutdown: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client)>,
    alert: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client, protocol: bindings::i2c_alert_protocol, data: u32)>,
    command: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client, cmd: u32, arg: *mut core::ffi::c_void) -> i32>,
    driver: bindings::device_driver,
    id_table: *const bindings::i2c_device_id,
    detect: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client, info: *mut bindings::i2c_board_info) -> i32>,
    address_list: Option<*const u16>,
    flags: Option<u32>,
}

impl I2CDriverBuilder {
    /// Creates a new `I2CDriverBuilder` instance.
    pub fn new(
        driver_name: *const i8,
        owner: *const bindings::module,
        probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
        remove: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
        id_table: *const bindings::i2c_device_id,
    ) -> Self {
        let mut driver: bindings::device_driver = unsafe { core::mem::zeroed() };
        driver.name = driver_name.as_ptr() as *const i8;
        driver.owner = owner;
        Self {
            class: None,
            probe,
            remove,
            shutdown: None,
            alert: None,
            command: None,
            driver,
            id_table,
            detect: None,
            address_list: None,
            flags: None,
        }
    }

    /// Sets the device class for the driver.
    pub fn class(mut self, class: u32) -> Self {
        self.class = Some(class);
        self
    }

    /// Sets the shutdown function for the driver.
    pub fn shutdown(mut self, shutdown: unsafe extern "C" fn(client: *mut bindings::i2c_client)) -> Self {
        self.shutdown = Some(shutdown);
        self
    }

    /// Sets the alert function for the driver.
    pub fn alert(mut self, alert: unsafe extern "C" fn(client: *mut bindings::i2c_client, protocol: bindings::i2c_alert_protocol, data: u32)) -> Self {
        self.alert = Some(alert);
        self
    }

    /// Sets the command function for the driver.
    pub fn command(mut self, command: unsafe extern "C" fn(client: *mut bindings::i2c_client, cmd: u32, arg: *mut core::ffi::c_void) -> i32) -> Self {
        self.command = Some(command);
        self
    }

    /// Sets the device detection function for the driver.
    pub fn detect(mut self, detect: unsafe extern "C" fn(client: *mut bindings::i2c_client, info: *mut bindings::i2c_board_info) -> i32) -> Self {
        self.detect = Some(detect);
        self
    }

    /// Sets the address list for device detection.
    pub fn address_list(mut self, address_list: *const u16) -> Self {
        self.address_list = Some(address_list);
        self
    }

    /// Sets the flags for the driver.
    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Sets the suspend function for the driver.
    pub fn suspend(mut self, suspend: unsafe extern "C" fn(client: *mut bindings::i2c_client, state: bindings::pm_message_t) -> i32) -> Self {
        self.suspend = Some(suspend);
        self
    }

    /// Sets the resume function for the driver.
    pub fn resume(mut self, resume: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32) -> Self {
        self.resume = Some(resume);
        self
    }

    /// Builds the `I2CDriver` instance.
    pub fn build(self) -> Result<I2CDriver> {
        let mut driver: bindings::i2c_driver = unsafe { core::mem::zeroed() };
        driver.driver = self.driver;
        driver.probe = Some(self.probe);
        driver.remove = Some(self.remove);
        driver.id_table = self.id_table;
        driver.class = self.class.unwrap_or(0);
        driver.shutdown = self.shutdown;
        driver.alert = self.alert;
        driver.command = self.command;
        driver.detect = self.detect;
        driver.address_list = self.address_list.unwrap_or(ptr::null());
        driver.flags = self.flags.unwrap_or(0);

        Ok(I2CDriver(unsafe { Opaque::new(&mut driver) }))
    }
}