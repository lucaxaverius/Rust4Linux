#![no_main]

use kernel::prelude::*;
use kernel::sync::{new_spinlock, SpinLock};
use core::str;
use alloc::vec::Vec;
use alloc::string::String;

extern "C" {
    fn create_device() -> i32;
    fn remove_device();
}

#[pin_data]
struct RulesContainer {
    #[pin]
    rules_lock: SpinLock<Vec<String>>,
}

impl RulesContainer {
    fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            rules_lock <- new_spinlock!(Vec::new()),
        })
    }

    fn add_rule(&self, rule: String) {
        let mut rules = self.rules_lock.lock();
        // Use GFP_KERNEL for general-purpose kernel memory allocations
        rules.push(rule, GFP_KERNEL).expect("Failed to push rule");    
    }

    fn get_rules(&self) -> String {
        let rules = self.rules_lock.lock();
        let mut result = String::new();

        for (i, rule) in rules.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(rule);
        }

        result.push('\n');
        result
    }

}

static mut RULES_CONTAINER: Option<Pin<RulesContainer>> = None;

fn init_rules() {
    unsafe {
        RULES_CONTAINER = Some(Box::pin_init(RulesContainer::new(), GFP_KERNEL)?);
    }
}

// Rust read function
#[no_mangle]
pub extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    buffer: *mut u8,
    len: usize,
    offset: *mut u64,
) -> isize {
    unsafe {
        if let Some(ref rules_container) = RULES_CONTAINER {
            let rules_str = rules_container.get_rules();
            let output_bytes = rules_str.as_bytes();
            let read_len = core::cmp::min(len, output_bytes.len());
            core::ptr::copy_nonoverlapping(output_bytes.as_ptr(), buffer, read_len);
            read_len as isize
        } else {
            0
        }
    }
}

// Rust write function
#[no_mangle]
pub extern "C" fn rust_write(
    _file: *mut core::ffi::c_void,
    buffer: *const u8,
    len: usize,
    _offset: *mut u64,
) -> isize {
    unsafe {
        let mut input = Vec::with_capacity(len);
        input.set_len(len);
        core::ptr::copy_nonoverlapping(buffer, input.as_mut_ptr(), len);
        match str::from_utf8(&input) {
            Ok(rule_str) => {
                if let Some(ref rules_container) = RULES_CONTAINER {
                    rules_container.add_rule(rule_str.to_string());
                }
                len as isize
            },
            Err(_) => -EINVAL as isize,
        }
    }
}

module! {
    type: SecModule,
    name: "sec_module",
    author: "Luca Saverio Esposito",
    description: "Security module using Rust and minimal C",
    license: "GPL",
}

struct SecModule;

impl kernel::Module for SecModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        init_rules();
        unsafe {
            if create_device() < 0 {
                return Err(Error::EINVAL);
            }
        }
        pr_info!("Security module loaded\n");
        Ok(SecModule)
    }
}

impl Drop for SecModule {
    fn drop(&mut self) {
        unsafe {
            remove_device();
        }
        pr_info!("Security module unloaded\n");
    }
}
