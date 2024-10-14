// i2c_test_driver.rs

// SPDX-License-Identifier: GPL-2.0

//! I2C Test Module
/// uses i2c rust api to interact with i2c-stub
use kernel::prelude::*;
use kernel::{ThisModule, bindings, i2c::*, str::CStr};
use kernel::{module_device_table};

module! {
    type: RustI2CDriver,
    name: "rust_i2c_driver",
    author: "Luca Saverio Esposito",
    description: "Rust I2C driver that uses i2c-stub to simulate a real hardware interaction",
    license: "GPL",
}

struct RustI2CDriver {
    driver: I2CDriver,
}

impl RustI2CDriver{
    /// The probe function that will interact with the device
    unsafe extern "C" fn probe_function(client: *mut bindings::i2c_client) -> i32 {
        pr_info!("Rust I2C driver probed for client at address 0x{:x}\n", (*client).addr);

        let device = unsafe{
            // Pass adapter and address
            I2CClient::from_raw_ptr(client)
        };    

        // Write a single byte to register 0x01
        if let Err(e) = device.write_byte(0x01, 0xAB) {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match device.read_byte(0x01) {
            Ok(value) => pr_info!("Read byte from register 0x01: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }

        // Write a single byte to register 0x01
        if let Err(e) = device.write_byte(0x01, 0xCC) {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match device.read_byte(0x01) {
            Ok(value) => pr_info!("Read byte from register 0x01: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }
        
        // Write a single byte to register 0x01
        if let Err(e) = device.write_byte(0x02, 0x12) {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match device.read_byte(0x02) {
            Ok(value) => pr_info!("Read byte from register 0x02: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }
        
        pr_info!("I2C device probed\n");

        0
    }
    
    unsafe extern "C" fn remove(client: *mut bindings::i2c_client) {
        pr_info!("Rust I2C driver removed for client at address 0x{:x}\n", client.addr());
    }
}

// Define the device ID table for the devices you want to support
static DEVICE_ID_TABLE: [bindings::i2c_device_id; 2] = [
    {I2CDeviceId::new(b"rust_i2c_dev",0).inner()},
    {I2CDeviceId::new(b"",0).inner()},
];

// Expose the device table to the kernel
module_device_table!(i2c, DEVICE_ID_TABLE);

static ADDRESS_LIST: [u16; 2] = [0x50, 0];  // 0x50 is the I2C address used by i2c-stub

impl kernel::Module for RustI2CDriver {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust I2C driver initializing\n");

        // Create a new I2C driver
        let driver_name = CStr::from_bytes_with_nul(b"rust_i2c_driver\0").unwrap().as_ptr() as *const i8;

        let builder = I2CDriverBuilder::new(
            driver_name,
            module.as_ptr(),
            Self::probe_function,
            Self::remove,
            DEVICE_ID_TABLE.as_ptr(),
        );

        let driver = builder.build()?;

        // Register the driver
        I2CDriver::add_driver(&driver)?;

        Ok(RustI2CDriver{driver})
    }
}

impl Drop for RustI2CDriver {
    fn drop(&mut self) {
        pr_info!("Rust I2C driver unloaded\n");
    }
}


