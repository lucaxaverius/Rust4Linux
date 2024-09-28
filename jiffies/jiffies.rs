// SPDX-License-Identifier: GPL-2.0

//! Rust abstractions for jiffies-related kernel functions.
//!
//! This module provides safe Rust bindings for the Linux kernel's timekeeping
//! functions related to jiffies. Jiffies are the units of time used by the kernel
//! to represent the passage of time, and these functions help in converting 
//! jiffies to more common time units such as milliseconds and microseconds.

//! C headers: [`include/linux/jiffies.h`](../../../../include/linux/jiffies.h)

use kernel::bindings;
use core::ffi::c_ulong;


/// Converts a given number of jiffies to milliseconds.
///
/// # Arguments
///
/// * `j` - The number of jiffies, provided as a 64-bit unsigned integer (`u64`).
///
/// # Returns
///
/// A `u64` representing the equivalent number of milliseconds.
///
/// # Safety
///
/// This function wraps a call to the kernel's `jiffies_to_msecs` function, ensuring
/// the argument type matches and is handled safely in Rust.
/// 
/// # Example
///
/// ```rust
/// let msecs = jiffies::jiffies_to_msecs(1000);
/// println!("Milliseconds: {}", msecs);
/// ```
pub fn jiffies_to_msecs(j: u64) -> u64 {
    // Call the unsafe kernel function within a safe Rust function.
    unsafe { bindings::jiffies_to_msecs(j as c_ulong) as u64 }
}

/// Converts a given number of jiffies to microseconds.
///
/// # Arguments
///
/// * `j` - The number of jiffies, provided as a 64-bit unsigned integer (`u64`).
///
/// # Returns
///
/// A `u64` representing the equivalent number of microseconds.
///
/// # Safety
///
/// This function wraps a call to the kernel's `jiffies_to_usecs` function, ensuring
/// the argument type matches and is handled safely in Rust.
///
/// # Example
///
/// ```rust
/// let usecs = jiffies::jiffies_to_usecs(1000);
/// println!("Microseconds: {}", usecs);
/// ```
pub fn jiffies_to_usecs(j: u64) -> u64 {
    // Call the unsafe kernel function within a safe Rust function.
    unsafe { bindings::jiffies_to_usecs(j as c_ulong) as u64 }
}
