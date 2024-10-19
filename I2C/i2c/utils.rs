// utils.rs

//! Utility functions and constants for I2C support.

use core::ffi::c_char;
use kernel::bindings;

/// Maximum size of an I2C device name.
pub const I2C_NAME_SIZE: usize = bindings::I2C_NAME_SIZE as usize;

/// Converts a byte slice into a fixed-size C string.
///
/// Ensures the string fits within `I2C_NAME_SIZE` and is null-terminated.
///
/// # Arguments
///
/// * `s` - The byte slice to convert.
///
/// # Returns
///
/// An array of `c_char` representing the C string.
pub const fn make_device_name(s: &[u8]) -> [c_char; I2C_NAME_SIZE] {
    let mut name = [0 as c_char; I2C_NAME_SIZE];
    let mut i = 0;
    while i < s.len() && i < I2C_NAME_SIZE - 1 {
        name[i] = s[i] as c_char;
        i += 1;
    }
    name
}
