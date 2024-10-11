// i2c_test_driver.rs

// SPDX-License-Identifier: GPL-2.0

//! I2C Driver Module
/// uses i2c rust api to interact with i2c-stub
use kernel::prelude::*;
use kernel::{ThisModule, bindings, i2c::*, str::CStr};

module! {
    type: SimpleI2CDriver,
    name: "simple_i2c_driver",
    author: "Luca Saverio Esposito",
    description: "Simple I2C driver that uses i2c-stub to simulate a real hardware interaction",
    license: "GPL",
}

struct SimpleI2CDriver {
    driver: I2CDriver,
}
// Define the device ID table for the devices you want to support
static DEVICE_IDS: [bindings::i2c_device_id; 2] = [
    bindings::i2c_device_id {
        name: [
            b's' as i8, b'i' as i8, b'm' as i8, b'p' as i8, b'l' as i8, b'e' as i8,
            b'_' as i8, b'i' as i8, b'2' as i8, b'c' as i8, b'_' as i8, b'd' as i8,
            b'e' as i8, b'v' as i8, b'i' as i8, b'c' as i8, b'e' as i8, 0, 0, 0
        ],
        driver_data: 0,
    },
    bindings::i2c_device_id {
        name: [0i8; 20], // Terminate the table
        driver_data: 0,
    },
];


static ADDRESS_LIST: [u16; 2] = [0x50, 0];  // 0x50 is the I2C address used by i2c-stub

impl kernel::Module for SimpleI2CDriver {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Simple I2C driver loaded\n");

        // Create and register the I2C driver
        let driver = I2CDriver::new(
            CStr::from_bytes_with_nul(b"simple_i2c_driver\0").unwrap().as_ptr() as *const i8,
            Some(probe_function),
            None, // Remove function can be added later
            module.as_ptr(),
            DEVICE_IDS.as_ptr(),
            ADDRESS_LIST.as_ptr(), 
        );
        unsafe { driver.register_driver(module.as_ptr())? };

        Ok(SimpleI2CDriver{driver})
    }
}

impl Drop for SimpleI2CDriver {
    fn drop(&mut self) {
        unsafe {
            self.driver.remove_driver();
        }
        pr_info!("Simple I2C driver unloaded\n");
    }
}

/// The probe function that will interact with the device
unsafe extern "C" fn probe_function(client: *mut bindings::i2c_client) -> i32 {
    let device = unsafe{
        // Pass adapter and address
        I2CClient::new((*client).adapter, (*client).addr) 
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
