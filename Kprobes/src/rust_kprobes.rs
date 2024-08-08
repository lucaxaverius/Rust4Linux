// SPDX-License-Identifier: GPL-2.0
// rust_kprobes.rs
//! Rust out-of-tree sample, with C code inclusion

use kernel::prelude::*;

const BLACKLISTED_USER_IDS: [u32; 3] = [1003, 1001, 1002];

module! {
    type: RustKprobes,
    name: "rust_kprobes",
    author: "Luca Saverio Esposito",
    description: "A simple test, trying to use kprobes in rust module with C help",
    license: "GPL",
}

struct RustKprobes;

impl kernel::Module for RustKprobes {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust kprobe module loaded!\n");

        // Call the C function to initialize kprobe
        call_initialize_kprobe();

        Ok(RustKprobes)
    }
}

impl Drop for RustKprobes {
    fn drop(&mut self) {
        pr_info!("Rust kprobe module unloaded!\n");

        // Call the C function to clean up kprobe
        call_cleanup_kprobe();
    }
}

// Rust function to check if a user ID is blacklisted
#[no_mangle]
pub extern "C" fn check_user_id(user_id: u32) -> bool {
    if BLACKLISTED_USER_IDS.contains(&user_id){
        pr_warn!("Kprobe: Blacklisted user {} detected!\n",user_id);    
        true
    }
    else{
        false
    }
}

// FFI declarations
extern "C" {
    fn initialize_kprobe();
    fn cleanup_kprobe();
}

pub fn call_initialize_kprobe() {
    unsafe { initialize_kprobe() }
}

pub fn call_cleanup_kprobe() {
    unsafe { cleanup_kprobe() }
}
