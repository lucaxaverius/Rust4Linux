// macros.rs

//! Module for I2C-related macros.
//!
//! This module provides macros to assist in driver development, such as generating
//! device tables and callback functions.

/// Exposes the device table to the kernel module loader.
///
/// Is similar to the `MODULE_DEVICE_TABLE` macro in C with a few more parameters.
///
/// # Parameters
///
/// * `$type_` - The device type identifier (e.g., `i2c`).
/// * `$name` - The name of your device ID table variable.
/// * `$device_id_type` - The full path to the device ID type.
/// * `$len` - The length of your device ID table array.
///
/// # Notes
///
/// - The macro exports a symbol with a name in the format:
///   `__mod_<type>__<name>_device_table`
/// - Can be wrapped inside device_type specific macro, like i2c_module_device_table.
#[macro_export]
macro_rules! module_device_table {
    ($type_:ident, $name:ident, $device_id_type:path, $len:expr) => {
        #[no_mangle]
        #[link_section = ".modinfo"]
        #[export_name = concat!(
            "__mod_",
            stringify!($type_),
            "__",
            stringify!($name),
            "_device_table"
        )]
        /// The array exposed to modinfo
        pub static __DEVICE_TABLE_ALIAS: [$device_id_type; $len] = $name;
    };
}

/// Exposes the I2C device table to the kernel module loader.
///
/// Converts an array of `I2CDeviceID` to an array of `bindings::i2c_device_id`
/// and exports it for driver matching.
///
/// # Parameters
///
/// * `$name` - The name of your `I2CDeviceID` table.
/// * `$len` - The length of your `I2CDeviceID` table array.
///
/// # Notes
/// - The converted table is named: __I2C_DEVICE_TABLE_BINDINGS, 
///   it must be used when building a new driver with I2CDriverBuilder.
#[macro_export]
macro_rules! i2c_module_device_table {
    ($name:ident, $len:expr) => {
        /// The static array of bindings generated from the I2C device ID table.
        /// This array is used to expose the device table to the kernel module loader.
        static __I2C_DEVICE_TABLE_BINDINGS: [kernel::bindings::i2c_device_id; $len] =
            crate::device_id::I2CDeviceID::to_bindings_array(&$name);

        // Expose the device table to the kernel module loader
        kernel::module_device_table!(
            i2c,
            __I2C_DEVICE_TABLE_BINDINGS,
            kernel::bindings::i2c_device_id,
            $len
        );
    };
}

/// Generates the unsafe extern "C" functions required for the I2C driver.
///
/// Creates the necessary C-compatible callback functions by calling the corresponding
/// methods from your Rust driver instance that implements `I2CDriverCallbacks`.
///
/// # Usage
///
/// ```rust
/// generate_i2c_callbacks!(MY_I2C_DRIVER_CALLBACKS);
/// ```
#[macro_export]
macro_rules! generate_i2c_callbacks {
    ($driver_instance:ident) => {
        /// Extern "C" probe callback that is triggered when the I2C device is being probed.
        /// 
        /// This function is automatically called by the kernel when a device that matches
        /// the driver's device ID table is detected on the I2C bus.
        #[no_mangle]
        pub unsafe extern "C" fn probe_callback(client: *mut kernel::bindings::i2c_client) -> i32 {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            match $driver_instance.probe(client) {
                Ok(_) => 0,
                Err(e) => e,
            }
        }

        /// Extern "C" remove callback that is triggered when the I2C device is removed.
        /// 
        /// This function is automatically called by the kernel when the I2C device is being
        /// removed from the bus or when the driver is unloaded.
        #[no_mangle]
        pub unsafe extern "C" fn remove_callback(client: *mut kernel::bindings::i2c_client) {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            $driver_instance.remove(client);
        }

        /// Extern "C" shutdown callback that is triggered when the system is shutting down.
        /// 
        /// This optional function can be provided to handle device-specific shutdown logic.
        #[no_mangle]
        pub unsafe extern "C" fn shutdown_callback(client: *mut kernel::bindings::i2c_client) {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            $driver_instance.shutdown(client);
        }

        /// Extern "C" alert callback that is triggered on I2C alerts.
        /// 
        /// This optional function is called when an I2C alert occurs, typically used in SMBus.
        #[no_mangle]
        pub unsafe extern "C" fn alert_callback(
            client: *mut kernel::bindings::i2c_client,
            protocol: kernel::bindings::i2c_alert_protocol,
            data: u32,
        ) {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            $driver_instance.alert(client, protocol, data);
        }

        /// Extern "C" command callback that is triggered for custom I2C commands.
        /// 
        /// This optional function allows custom commands to be sent to the I2C device.
        #[no_mangle]
        pub unsafe extern "C" fn command_callback(
            client: *mut kernel::bindings::i2c_client,
            cmd: u32,
            arg: *mut core::ffi::c_void,
        ) -> i32 {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            match $driver_instance.command(client, cmd, arg) {
                Ok(_) => 0,
                Err(e) => e,
            }
        }

        /// Extern "C" detect callback that is triggered for device detection on the I2C bus.
        /// 
        /// This optional function can be used to detect devices on the I2C bus that do not
        /// explicitly announce their presence.
        #[no_mangle]
        pub unsafe extern "C" fn detect_callback(
            client: *mut kernel::bindings::i2c_client,
            info: *mut kernel::bindings::i2c_board_info,
        ) -> i32 {
            let client = kernel::i2c::I2CClient::from_raw_ptr(client);
            match $driver_instance.detect(client, info) {
                Ok(_) => 0,
                Err(e) => e,
            }
        }
    };
}
