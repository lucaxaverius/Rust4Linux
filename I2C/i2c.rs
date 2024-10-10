// SPDX-License-Identifier: GPL-2.0

//! I2C support.
//!
//! This module contains the kernel APIs related to I2C that have b>
//! wrapped for usage by Rust code in the kernel.

use kernel::prelude::*;
use kernel::bindings;
use kernel::error::{Error, Result, to_result};

use crate::{init::PinInit,types::Opaque,pin_init};
use core::ptr;


/// This structure represent the C `i2c_msg` struct.
/// It is the low level representation of one segment of an I2C transaction.
#[repr(C)]
pub struct I2CMsg {
    addr: u16,
    flags: u16,
    len: u16,
    buf: *mut u8,
}

impl I2CMsg {
    /// Creates a new `I2CMsg` instance.
    /// 
    /// # Arguments
    /// * `addr` - Slave address, either 7 or 10 bits.
    /// * `flags` - Flags for the message.
    /// * `buf` - Buffer to read from or write to.
    /// 
    /// # Returns
    /// * `I2CMsg` instance with the provided parameters.
    /// 
    /// # Safety
    /// The caller must ensure that the buffer remains valid while the `I2CMsg` is in use.
    pub fn new(addr: u16, flags: u16, buf: &mut [u8]) -> Self {
        I2CMsg {
            addr,
            flags,
            len: buf.len() as u16,
            buf: buf.as_mut_ptr(),
        }
    }

    /// I2C message flag for reading data (from slave to master).
    pub const I2C_M_RD: u16 = 0x0001;
    /// I2C message flag for 10-bit chip address.
    pub const I2C_M_TEN: u16 = 0x0010;
    /// I2C message flag indicating the buffer is DMA safe.
    pub const I2C_M_DMA_SAFE: u16 = 0x0200;
    /// I2C message flag indicating the message length will be first received byte.
    pub const I2C_M_RECV_LEN: u16 = 0x0400;
    /// I2C message flag to skip the ACK/NACK bit in read message.
    pub const I2C_M_NO_RD_ACK: u16 = 0x0800;
    /// I2C message flag to treat NACK from client as ACK.
    pub const I2C_M_IGNORE_NAK: u16 = 0x1000;
    /// I2C message flag to toggle the Rd/Wr bit.
    pub const I2C_M_REV_DIR_ADDR: u16 = 0x2000;
    /// I2C message flag to skip repeated start sequence.
    pub const I2C_M_NOSTART: u16 = 0x4000;
    /// I2C message flag to force a STOP condition after the message.
    pub const I2C_M_STOP: u16 = 0x8000;

    /// Returns a reference to the buffer as a slice. It can be used to read the buffer.
    /// 
    /// # Safety
    /// The caller must ensure that `buf` is a valid pointer and `len` is accurate.
    pub fn read_from_buf(&self) -> &[u8] {
        // SAFETY: Caller must ensure buffer and length are valid.
        unsafe { alloc::slice::from_raw_parts(self.buf, self.len as usize) }
    }

    /// Returns a mutable reference to the buffer as a slice. It can be used to read and write the buffer.
    /// 
    /// # Safety
    /// The caller must ensure that `buf` is a valid pointer and `len` is accurate.
    pub fn write_to_buf(&mut self) -> &mut [u8] {
        // SAFETY: Caller must ensure buffer and length are valid.
        unsafe { alloc::slice::from_raw_parts_mut(self.buf, self.len as usize) }
    }
}

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
            Err(EINVAL)
        } else {
            // Safety: The pointer is non-null and valid, so we can use ffi_init to safely initialize the Opaque instance.
            pin_init!(I2CAdapter {
                // SAFETY: We have verified that `adapter_ptr` is non-null and valid.
                0 <- Opaque::ffi_init(|opaque_ptr| unsafe {
                    core::ptr::copy_nonoverlapping(adapter_ptr, opaque_ptr, 1);
                }),
            })    
        }
    }


    /// Executes an I2C transfer.
    /// 
    /// # Arguments
    /// * `msgs` - A slice of `I2CMsg` instances representing the messages to be transferred.
    /// 
    /// # Returns
    /// * `Ok(usize)` if the transfer is successful, indicating the number of messages transferred.
    /// * `Err(Error)` if an error occurs during the transfer.
    pub fn transfer(&self, msgs: &mut [I2CMsg]) -> Result<usize> {
        let ret = unsafe {
            bindings::i2c_transfer(
                self.0.get(),
                msgs.as_mut_ptr() as *mut bindings::i2c_msg,
                msgs.len() as i32,
            )
        };
        to_result(ret).map(|_| ret as usize)
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
        let client_ptr = unsafe { bindings::i2c_new_client_device(adapter.0.get(), board_info.0.get()) };

        if client_ptr.is_null() || (client_ptr as isize) < 0 {
            Err(EINVAL)
        } else {
            // Safety: The pointer is non-null and valid, so it's safe to create an I2CClient instance.
            pin_init!(I2CClient {
                // SAFETY: We have verified that `client_ptr` is non-null and valid.
                0 <- Opaque::ffi_init(|opaque_ptr| unsafe {
                    core::ptr::copy_nonoverlapping(client_ptr, opaque_ptr, 1);
                }),
            })   
          }
    }


    /// Sends a single message to the I2C client device.
    /// 
    /// # Arguments
    /// * `buf` - A buffer containing the data to be sent. Must be less than 2^16 since msg.len is u16.
    /// 
    /// # Returns
    /// * `Ok(usize)` if the data is successfully sent, indicating the number of bytes written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn master_send(&self, buf: &[u8]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_send(self.0.get(), buf.get() as *const i8, buf.len() as i32) };
        // this.map is used to convert the OK() to usize
        to_result(ret).map(|_| ret as usize)
    }

    /// Receives data from the I2C client device.
    /// 
    /// # Arguments
    /// * `buf` - A buffer to store the received data. Must be less than 2^16 since msg.len is u16.
    /// 
    /// # Returns
    /// * `Ok(usize)` if the data is successfully received, indicating the number of bytes read.
    /// * `Err(Error)` if an error occurs during reception.
    pub fn master_recv(&self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_recv(self.0.get(), buf.as_mut_ptr() as *mut i8, buf.len() as i32) };
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
        let ret = unsafe { bindings::i2c_smbus_write_byte(self.0.get(), value) };
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
        let ret = unsafe { bindings::i2c_smbus_read_byte(self.0.get()) };
        if ret < 0 {
            to_result(ret)
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
        let ret = unsafe { bindings::i2c_smbus_write_byte_data(self.0.get(), command as i32, value as i32) };
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
        let ret = unsafe { bindings::i2c_smbus_read_byte_data(self.0.get(), command as i32) };
        if ret < 0 {
            to_result(ret)
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
        let ret = unsafe { bindings::i2c_smbus_write_word_data(self.0.get(), command as i32, value as i32) };
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
        let ret = unsafe { bindings::i2c_smbus_read_word_data(self.0.get(), command as i32) };
        if ret < 0 {
            to_result(ret)
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
                self.0.get(),
                command as i32,
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
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` if the block is successfully read.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn read_block(&self, command: u8) -> Result<Vec<u8>> {
        let mut values = [0u8; 32];
        let ret = unsafe {
            bindings::i2c_smbus_read_block_data(
                self.0.get(),
                command as i32,
                values.as_mut_ptr() as *mut i8,
            )
        };
        if ret < 0 {
            to_result(ret)
        } else {
            Ok(values[..ret as usize].to_vec())
        }
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
        let ret = unsafe { bindings::i2c_add_driver(driver.0.get()) };
        to_result(ret)
    }

    /// Unregisters the I2C driver from the kernel.
    /// 
    /// # Arguments
    /// * `driver` - The I2C driver to be unregistered.
    pub fn del_driver(driver: &I2CDriver) {
        unsafe { bindings::i2c_del_driver(driver.0.get()) };
    }
}

/// Implement Send and Sync for I2CDriver
unsafe impl Send for I2CDriver {}
unsafe impl Sync for I2CDriver {}


/// A builder for creating an `I2CDriver` instance.
pub struct I2CDriverBuilder {
    class: Option<u32>,
    probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
    remove: unsafe extern "C" fn(client: *mut bindings::i2c_client),
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
        driver.name = driver_name.as_ptr() as *const u8;
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