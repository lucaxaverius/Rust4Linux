// driver.rs

//! Module for I2C driver representation.
//!
//! This module provides structures and traits for creating and managing I2C drivers.

use kernel::prelude::*;
use kernel::bindings;
use core::ffi::{c_void,c_int};
use crate::i2c::client::I2CClient;
use crate::error::to_result;

/// Represents an I2C driver.
///
/// An `I2CDriver` contains the necessary information to register and manage an I2C driver in the kernel.
pub struct I2CDriver {
    /// Pointer to the underlying `i2c_driver` struct.
    driver: *mut bindings::i2c_driver,
}

impl I2CDriver {
    /// Registers the I2C driver with the kernel.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registration is successful.
    /// * `Err(Error)` if registration fails.
    pub fn add_driver(&self) -> Result<()> {
        if self.driver.is_null() {
            return Err(EINVAL);
        } 
        let ret = unsafe { bindings::i2c_add_driver(self.driver) };
        to_result(ret)
    }

    /// Deregisters the I2C driver from the kernel and free the heap.
    ///
    /// It must be called in the Drop trait of the kernel module.
    pub fn remove_driver(&self) {
        if self.driver.is_null() {
            pr_info!("WARNING!!! Called remove driver to null ptr !!!");
            return;
        } 
        unsafe { 
            bindings::i2c_del_driver(self.driver);
            // Convert the raw pointer back to a Box so that Rust can properly deallocate it

            drop(Box::from_raw(self.driver));
        };
    
    }
}

unsafe impl Send for I2CDriver {}
unsafe impl Sync for I2CDriver {}

/// Builder for creating an `I2CDriver` instance.
///
/// Provides a convenient way to construct an `I2CDriver` with optional parameters.
pub struct I2CDriverBuilder {
    // Fields for driver configuration
    class: Option<u32>,
    probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
    remove: unsafe extern "C" fn(client: *mut bindings::i2c_client),
    shutdown: Option<unsafe extern "C" fn(client: *mut bindings::i2c_client)>,
    alert: Option<
        unsafe extern "C" fn(
            client: *mut bindings::i2c_client,
            protocol: bindings::i2c_alert_protocol,
            data: u32,
        ),
    >,
    command: Option<
        unsafe extern "C" fn(
            client: *mut bindings::i2c_client,
            cmd: u32,
            arg: *mut core::ffi::c_void,
        ) -> i32,
    >,
    driver: bindings::device_driver,
    id_table: *const bindings::i2c_device_id,
    detect: Option<
        unsafe extern "C" fn(
            client: *mut bindings::i2c_client,
            info: *mut bindings::i2c_board_info,
        ) -> i32,
    >,
    address_list: Option<*const u16>,
    clients: Option<bindings::list_head>,
    flags: Option<u32>,
}

impl I2CDriverBuilder {
    /// Creates a new `I2CDriverBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `driver_name` - Name of the driver.
    /// * `owner` - Pointer to the module owning this driver.
    /// * `probe` - Probe callback function.
    /// * `remove` - Remove callback function.
    /// * `id_table` - Pointer to the device ID table.
    pub fn new(
        driver_name: *const i8,
        owner: *mut bindings::module,
        probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
        remove: unsafe extern "C" fn(client: *mut bindings::i2c_client),
        id_table: *const bindings::i2c_device_id,
    ) -> Self {
        Self {
            driver: bindings::device_driver {
                name: driver_name,
                owner,
                ..Default::default()
            },
            class: None,
            probe,
            remove,
            shutdown: None,
            alert: None,
            command: None,
            id_table,
            detect: None,
            address_list: None,
            clients: None,
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
    pub fn clients(mut self, clients: bindings::list_head) -> Self {
        self.clients = Some(clients);
        self
    }

    /// Sets the flags for the driver.
    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = Some(flags);
        self
    }
    
    /// Builds and returns an `I2CDriver` instance.
    ///
    /// # Returns
    ///
    /// * `Ok(I2CDriver)` if the driver is successfully built.
    /// * `Err(Error)` if driver creation fails.
    pub fn build(self) -> Result<I2CDriver> {
        let driver = bindings::i2c_driver {
            driver: self.driver,
            probe: Some(self.probe),
            remove: Some(self.remove),
            id_table: self.id_table,
            class: self.class.unwrap_or(0),
            shutdown: self.shutdown,
            alert: self.alert,
            command: self.command,
            detect: self.detect,
            address_list: self.address_list.unwrap_or(core::ptr::null()),
            clients: self.clients.unwrap_or(bindings::list_head {
                next: core::ptr::null_mut(),
                prev: core::ptr::null_mut(),
            }),
            flags: self.flags.unwrap_or(0),
        };

          
        // Boxes provide ownership for this allocation, and drop their contents when they go out of scope
        let driver_ptr = Box::into_raw(Box::new(driver,GFP_KERNEL).expect("Driver allocation failed"));

        Ok(I2CDriver { driver: driver_ptr })
    }
}

/// Trait representing the essential callbacks for an I2C driver.
///
/// Implement this trait to define the behavior of your I2C driver.
///
/// # Safety:
///
/// The `I2CDriverCallbacks` trait is required to implement both `Send` and `Sync`, to be implemented in a static context.
/// Implementors of this trait are responsible for ensuring that their internal state adheres to
/// Rust's concurrency guarantees, making the `Send + Sync` markers appropriate.
///
pub trait I2CDriverCallbacks: Send + Sync {
    /// Called when the driver is bound to an I2C device.
    ///
    /// # Arguments
    ///
    /// * `client` - The `I2CClient` representing the device.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if probing is successful.
    /// * `Err(c_int)` if probing fails.
    fn probe(&self, client: I2CClient) -> Result<(), c_int>;

    /// Called when the driver is unbound from an I2C device.
    ///
    /// # Arguments
    ///
    /// * `client` - The `I2CClient` representing the device.
    fn remove(&self, client: I2CClient);

    /// Optional: Called during device shutdown.
    ///
    /// Default implementation does nothing.
    fn shutdown(&self, _client: I2CClient) {
        pr_info!("I2C Shutdown called\n");
    }

    /// Optional: Called on I2C alerts.
    ///
    /// Default implementation does nothing.
    fn alert(
        &self,
        _client: I2CClient,
        _protocol: bindings::i2c_alert_protocol,
        _data: u32,
    ) {
        pr_info!("I2C Alert called\n");
    }

    /// Optional: Handles custom I2C commands.
    ///
    /// Default implementation does nothing.
    fn command(
        &self,
        _client: I2CClient,
        _cmd: u32,
        _arg: *mut c_void,
    ) -> Result<(), c_int> {
        pr_info!("I2C Command called\n");
        Ok(())
    }

    /// Optional: Performs device detection.
    ///
    /// Default implementation does nothing.
    fn detect(
        &self,
        _client: I2CClient,
        _info: *mut bindings::i2c_board_info,
    ) -> Result<(), c_int> {
        pr_info!("I2C Detect called\n");
        Ok(())
    }
}
