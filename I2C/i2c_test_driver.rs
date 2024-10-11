// i2c_test_driver.rs

// SPDX-License-Identifier: GPL-2.0

//! I2C Test Module
/// uses i2c rust api to interact with i2c-stub
use kernel::prelude::*;
use kernel::{ThisModule, bindings, i2c::*, str::CStr};

module! {
    type: RustI2CDriver,
    name: "rust_i2c_driver",
    author: "Luca Saverio Esposito",
    description: "Rust I2C driver that uses i2c-stub to simulate a real hardware interaction",
    license: "GPL",
}

static struct RustI2CDriver {
    driver: I2CDriver,
}

imp RustI2CDriver{
    /// The probe function that will interact with the device
    unsafe extern "C" fn probe_function(client: *mut bindings::i2c_client) -> i32 {
        pr_info!("Rust I2C driver probed for client at address 0x{:x}\n", (*client).addr);

        let device = unsafe{
            // Pass adapter and address
            I2CClient::from_raw_ptr(client)
        };    

        // Write a single byte to register 0x01
        if let Err(e) = unsafe{device.write_byte(0x01, 0xAB)} {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match unsafe{device.read_byte(0x01)} {
            Ok(value) => pr_info!("Read byte from register 0x01: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }

        // Write a single byte to register 0x01
        if let Err(e) = unsafe{device.write_byte(0x01, 0xCC)} {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match unsafe{device.read_byte(0x01)} {
            Ok(value) => pr_info!("Read byte from register 0x01: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }
        
        // Write a single byte to register 0x01
        if let Err(e) = unsafe{device.write_byte(0x02, 0x12)} {
            pr_err!("Failed to write byte: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read back the byte from the same register
        match unsafe{device.read_byte(0x02)} {
            Ok(value) => pr_info!("Read byte from register 0x02: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte: {:?}\n", e),
        }
        
        pr_info!("I2C device probed\n");

        0
    }



}

// Define the device ID table for the devices you want to support
static DEVICE_ID_TABLE: [I2CDeviceId; 2] = [
    I2CDeviceId {
        name: *b"rust_i2c_dev\0" as *const c_char,
        driver_data: 0,
    },
    I2CDeviceId {
        name: ptr::null(), // Terminate the table
        driver_data: 0,
    },
];

// Expose the device table to the kernel
module_device_table!(i2c, DEVICE_ID_TABLE);

static ADDRESS_LIST: [u16; 2] = [0x50, 0];  // 0x50 is the I2C address used by i2c-stub

impl kernel::Module for RustI2CDriver {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust I2C driver initializing\n");

        // Create a new I2C driver
        let driver_name = c_str!("rust_i2c_driver");

        let builder = I2CDriverBuilder::new(
            driver_name.as_bytes_with_nul(),
            kernel::THIS_MODULE,
            Self::probe,
            Self::remove,
            DEVICE_ID_TABLE.as_ptr(),
        );

        let i2c_driver = builder.build()?;

        // Register the driver
        I2CDriver::add_driver(&i2c_driver)?;

        Ok(RustI2CDriver{driver})
    }
}

impl Drop for RustI2CDriver {
    fn drop(&mut self) {
        unsafe {
            self.driver.remove_driver();
        }
        pr_info!("Rust I2C driver unloaded\n");
    }
}


