// i2c.rs

//! I2C support module.
//!
//! This module provides abstractions and utilities for interacting with I2C devices in the Linux kernel.
//! It is organized into several submodules, each handling different aspects of I2C communication.

pub mod msg;
pub mod adapter;
pub mod board_info;
pub mod device_id;
pub mod client;
pub mod driver;
pub mod macros;
pub mod utils;

// Re-exporting the main types and macros for ease of use
pub use msg::I2CMsg;
pub use adapter::I2CAdapter;
pub use board_info::I2CBoardInfo;
pub use device_id::I2CDeviceID;
pub use client::I2CClient;
pub use driver::{I2CDriver, I2CDriverBuilder, I2CDriverCallbacks};

#[allow(unused_imports)]
#[allow(unreachable_pub)]
pub use macros::*;

pub use utils::I2C_NAME_SIZE;
