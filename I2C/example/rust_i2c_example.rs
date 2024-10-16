// i2c_test_driver.rs

// SPDX-License-Identifier: GPL-2.0

//! I2C Test Module
/// uses i2c rust api to interact with i2c-stub
use kernel::prelude::*;
use kernel::{ThisModule, bindings, i2c::*, str::CStr};
use kernel::{module_device_table};

module! {
    type: RustI2CDriver,
    name: "rust_i2c_example",
    author: "Luca Saverio Esposito",
    description: "Rust I2C driver the functionalities of i2c.rs crate",
    license: "GPL",
}

struct RustI2CDriver {
    driver: I2CDriver,
}

impl RustI2CDriver{
    /// The probe function that will interact with the device
    unsafe extern "C" fn probe_function(client: *mut bindings::i2c_client) -> i32 {
        pr_info!("Rust I2C driver probed\n");

        // Pass adapter and address
        let device = I2CClient::new_client_device(,BOARD_INFO);
            
        // Write a byte to register 0x01
        if let Err(e) = device.write_byte(0x01, 0xAB) {
            pr_err!("Failed to write byte to register 0x01: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read a byte from register 0x01
        match device.read_byte(0x01) {
            Ok(value) => pr_info!("Read byte from register 0x01: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read byte from register 0x01: {:?}\n", e),
        }
        
        // Write a word to register 0x02
        if let Err(e) = device.write_word(0x02, 0x1234) {
            pr_err!("Failed to write word to register 0x02: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read a word from register 0x02
        match device.read_word(0x02) {
            Ok(value) => pr_info!("Read word from register 0x02: 0x{:X}\n", value),
            Err(e) => pr_err!("Failed to read word from register 0x02: {:?}\n", e),
        }

        // Write a block of data to register 0x03
        let data_to_write = [0x01, 0x02, 0x03, 0x04];
        if let Err(e) = device.write_block(0x03, &data_to_write) {
            pr_err!("Failed to write block to register 0x03: {:?}\n", e);
            return -EINVAL.to_errno();
        }

        // Read a block of data from register 0x03
        let mut read_buffer = [0u8; 4];
        match device.read_block(0x03, &mut read_buffer) {
            Ok(bytes_read) => pr_info!("Read block from register 0x03: {:X?}, bytes read: {}\n", &read_buffer[..bytes_read], bytes_read),
            Err(e) => pr_err!("Failed to read block from register 0x03: {:?}\n", e),
        }

        pr_info!("I2C device probed\n");

        0
    }

    unsafe extern "C" fn remove_function(_client: *mut bindings::i2c_client) {
        
        //pr_info!("Rust I2C driver removed for client at address 0x{:x}\n", (*client).addr);
        pr_info!("Rust I2C says bye do the device\n");
        
    }
}

// Define the device ID table for the devices you want to support
static ID_TABLE: [bindings::i2c_device_id; 3] = [
    {I2CDeviceIDArray::new_record(b"rust_i2c_dev",0)},
    {I2CDeviceIDArray::new_record(b"rust_i2c_dev_n2",0)},
    {I2CDeviceIDArray::new_record(b"",0)},
];

// Expose the device table to the kernel module loader
module_device_table!(i2c, ID_TABLE, bindings::i2c_device_id, 3);
// To check if the aliases are correctly loaded:
// modinfo ./rust_i2c_driver.ko | grep alias

static DEVICE_ID_TABLE: I2CDeviceIDArray = I2CDeviceIDArray::new(ID_TABLE.as_ptr());

static ADDRESS_LIST: [u16; 2] = [0x50, 0];  // 0x50 is the I2C address used by i2c-stub

static BOARD_INFO: I2CBoardInfo = I2CBoardInfo::new(b"rust_i2c_driver\0",50);


impl kernel::Module for RustI2CDriver {
    fn init(module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust I2C driver initializing\n");

        let adapter = I2CAdapter::get_from_bus_number(1);
        
        let builder : I2CDriverBuilder;

        if (!adapter.is_null()){
            // Create a new I2C driver
            let driver_name = CStr::from_bytes_with_nul(b"rust_i2c_driver\0").unwrap().as_ptr() as *const i8;

            builder = I2CDriverBuilder::new(
                driver_name,
                module.as_ptr(),
                Self::probe_function,
                Self::remove_function,
                DEVICE_ID_TABLE.as_ptr(),
            )
            .address_list(ADDRESS_LIST.as_ptr());
        }

        // Build driver structure
        let driver = builder.build()?;
        // Register the driver
        driver.add_driver()?;
     
        Ok(RustI2CDriver{driver,})
    

    }
}

impl Drop for RustI2CDriver {
    fn drop(&mut self) {
        self.driver.remove_driver();
        pr_info!("Rust I2C driver unloaded\n");
    }
}


