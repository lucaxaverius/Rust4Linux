// SPDX-License-Identifier: GPL-2.0
// rust_kprobes.rs
//! Rust out-of-tree sample

#![no_std]
#![no_main]

use kernel::prelude::*;

module! {
    type: RustOutOfTree,
    name: "Rust Kprobes for kernel hacking",
    author: "Luca Saverio Esposito",
    description: "A simple test, trying to use kprobes in rust module with C help",
    license: "GPL",
}

struct Module;

impl kernel::Module for Module {
    fn init() -> Result<Self> {
        pr_info!("Rust kprobe module loaded!\n");

        // Call the C function to initialize kprobe
        unsafe { initialize_kprobe() };

        Ok(Module)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        pr_info!("Rust kprobe module unloaded!\n");

        // Call the C function to clean up kprobe
        unsafe { cleanup_kprobe() };
    }
}

// FFI declarations
extern "C" {
    fn initialize_kprobe();
    fn cleanup_kprobe();
}
