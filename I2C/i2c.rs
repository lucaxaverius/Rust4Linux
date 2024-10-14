// SPDX-License-Identifier: GPL-2.0

//! I2C support.
//!
//! This module contains the kernel APIs related to I2C that have been
//! wrapped for usage by Rust code in the kernel.

use kernel::prelude::*;
use kernel::bindings;
use kernel::error::{Error, Result, to_result};
use crate::{types::Opaque};
//use crate::{init::PinInit,pin_init};
use core::ptr;
use core::ffi::c_char;


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
pub struct I2CAdapter {
    ptr: *mut bindings::i2c_adapter,
}

impl I2CAdapter {
    /// Attempts to get an I2C adapter from a given bus number.
    /// Effectively owns a reference to the adapter
    /// 
    /// # Arguments
    /// * `bus_number` - The bus number for which the I2C adapter is requested.
    /// 
    /// # Returns
    /// * `Ok(I2CAdapter)` if an adapter is found.
    /// * `Err(Error)` if the adapter cannot be found (i.e., null pointer returned).
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
                self.ptr,
                msgs.as_mut_ptr() as *mut bindings::i2c_msg,
                msgs.len() as i32,
            )
        };
        to_result(ret).map(|_| ret as usize)
    }
}

impl Drop for I2CAdapter {
    fn drop(&mut self) {
        unsafe { bindings::i2c_put_adapter(self.ptr) };
    }
}

/// This structure wraps the C `i2c_board_info` struct.
/// It is used to build tables of information listing I2C devices
/// that are present. This information is used to grow the driver model tree.
#[repr(transparent)]
pub struct I2CBoardInfo {
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


impl I2CBoardInfo{
    /// Creates a new `I2CBoardInfo` instance.
    /// It is the Rust counter part of the c macro I2C_BOARD_INFO
    ///
    /// # Arguments
    ///
    /// * `dev_type` - The device type identifier as a byte slice (without null terminator).
    /// * `dev_addr` - The device address on the bus.
    ///
    /// # Returns
    ///
    /// An instance of `I2CBoardInfo`.
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
            ..ZEROED_I2C_BOARD_INFO // Rest of the fields are zeroed
        };

        I2CBoardInfo { inner }
    }
    /// Returns a copy of the inner `bindings::i2c_board_info` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned struct is used correctly and that the `I2CBoardInfo`
    /// instance remains valid as long as the data is being used.
    pub const fn inner(&self) -> bindings::i2c_board_info{
        self.inner
    }
    /// Returns a pointer to the inner `i2c_board_info` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned pointer is used correctly and that
    /// the `I2CBoardInfo` instance remains valid as long as the pointer is in use.
    pub fn as_ptr(&self) -> *const bindings::i2c_board_info {
        &self.inner as *const bindings::i2c_board_info
    }

}

unsafe impl Sync for I2CBoardInfo {}
unsafe impl Send for I2CBoardInfo {}


/// This structure wraps the C `i2c_device_id` struct.
/// The last record of the struct has "" as name and 0 as data.
#[repr(transparent)]
pub struct I2CDeviceIDArray {
    /// contains the underlying c struct.
    inner: *bindings::i2c_device_id,
}

/// Max size of I2C device name 
pub const I2C_NAME_SIZE: usize = bindings::I2C_NAME_SIZE as usize;

/// Utility function that converts an array of u8 into a c_char[I2C_NAME_SIZE];
const fn make_device_name(s: &[u8]) -> [c_char; I2C_NAME_SIZE] {
    let mut name = [0 as c_char; I2C_NAME_SIZE];
    let mut i = 0;
    while i < s.len() && i < I2C_NAME_SIZE - 1 {
        name[i] = s[i] as c_char;
        i += 1;
    }
    name
}

impl I2CDeviceIDArray {
    /// Creates a new `I2CDeviceId` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the I2C device as a byte slice (`&[u8]`). This will be converted into a
    ///   null-terminated C string representing the device type. The name should not exceed
    ///   `I2C_NAME_SIZE - 1` (19 bytes) to leave room for the null terminator.
    /// * `driver_data` - The driver-specific data for the I2C device. This field is used to pass
    ///   additional information that may be needed by the driver.
    ///
    /// # Returns
    ///
    /// An instance of `I2CDeviceId`.
    ///
    /// # Example
    ///
    /// ```
    /// use kernel::i2c::I2CDeviceId;
    ///
    /// const DEVICE_ID: I2CDeviceId = I2CDeviceId::new(b"example_device", 0);
    /// ```
    ///
    /// # Panics
    ///
    /// This function does not panic. If the provided `name` exceeds `I2C_NAME_SIZE - 1` bytes,
    /// it will be truncated to fit.
    pub const fn new(name: &[u8], driver_data: u64) -> bindings::i2c_device_id {
       
        let name_array = make_device_name(name);

        // Create a new `i2c_device_id` struct and fill in the fields.
        let inner = bindings::i2c_device_id {
            name: name_array,
            driver_data,
        };

        I2CDeviceId { inner }
    }

    /// Returns a copy of the inner `bindings::i2c_device_id` struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the returned struct is used correctly and that the `I2CDeviceId`
    /// instance remains valid as long as the data is being used.
    pub const fn inner(&self) -> bindings::i2c_device_id {
        self.inner
    }
}

unsafe impl Sync for I2CDeviceId {}
unsafe impl Send for I2CDeviceId {}


/// Equivalent to the `MODULE_DEVICE_TABLE` macro in C.
///
/// Exposes the device table to the kernel module loader by creating a symbol
/// with a specific name that `modpost` can find.
///
/// # Arguments
///
/// * `$type_` - The type of the device table (e.g., `i2c`).
/// * `$name` - The name of the device table variable.
///
/// # Example
///
/// ```rust
/// module_device_table!(i2c, DEVICE_ID_TABLE);
/// ```
#[macro_export]
macro_rules! module_device_table {
    ($type_:ident, $name:ident) => {
        #[no_mangle]
        #[export_name = concat!("__mod_", stringify!($type_), "__", stringify!($name), "_device_table")]
        pub static __DEVICE_TABLE_ALIAS: *const $crate::i2c::I2CDeviceId = &$name as *const _;
    };
}





/// This structure wraps the C `i2c_client`, it represents an I2C slave device (i.e. chip)
/// connected to any i2c bus. 
///
/// The behaviour exposed to Linux is defined by the driver managing the device.
pub struct I2CClient {
    ptr: *mut bindings::i2c_client,
    owned: bool,
}

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
    pub fn new_client_device(adapter: &I2CAdapter, board_info: &I2CBoardInfo) -> Result<Self> {
        // Safety: Calling the C API `i2c_new_client_device` which returns a pointer to `i2c_client` or an error pointer.
        let client_ptr = unsafe { bindings::i2c_new_client_device(adapter.ptr, board_info.as_ptr()) };

        if client_ptr.is_null() || (client_ptr as isize) < 0 {
            Err(EINVAL)
        } else {
            // Safety: The pointer is non-null and valid, so it's safe to create an I2CClient instance.
            Ok(Self { ptr: client_ptr, owned: true })
          }
    }

    /// Creates a borrowed (managed by the kernel) `I2CClient` instance from a raw pointer.
    /// 
    /// # Arguments
    /// * `client_ptr` - A raw pointer to an `i2c_client` struct.
    /// 
    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a properly initialized `i2c_client`
    pub unsafe fn from_raw_ptr(ptr: *mut bindings::i2c_client) -> Self {
        Self { ptr, owned: false }
    }


    /// Sends a single message to the I2C client device.
    /// 
    /// # Arguments
    /// * `buf` - A buffer containing the data to be sent. Must be less than 2^16 since msg.len is u16.
    /// 
    /// # Returns
    /// * `Ok(usize)` if the data is successfully sent, indicating the number of bytes written.
    /// * `Err(Error)` if an error occurs during transmission.
    pub fn master_send(&self, buf: &[c_char]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_send(self.ptr, buf.as_ptr(), buf.len() as i32) };
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
    pub fn master_recv(&self, buf: &mut [c_char]) -> Result<usize> {
        if buf.len() > u16::MAX as usize {
            return Err(EINVAL);
        }
        let ret = unsafe { bindings::i2c_master_recv(self.ptr, buf.as_mut_ptr(), buf.len() as i32) };
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
            unsafe { bindings::i2c_unregister_device(self.ptr) };
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
}

impl Drop for I2CDriver{
    fn drop(&mut self) {
        // Unregisters the I2C driver from the kernel.
        unsafe { bindings::i2c_del_driver(self.0.get()) };
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
        owner: *mut bindings::module,
        probe: unsafe extern "C" fn(client: *mut bindings::i2c_client) -> i32,
        remove: unsafe extern "C" fn(client: *mut bindings::i2c_client),
        id_table: *const bindings::i2c_device_id,
    ) -> Self {
        let mut driver: bindings::device_driver = unsafe { core::mem::zeroed() };
        driver.name = driver_name;
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

        Ok(I2CDriver(Opaque::new(driver)))
    }
}