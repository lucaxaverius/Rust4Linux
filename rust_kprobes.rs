// SPDX-License-Identifier: GPL-2.0
// rust_kprobes.rs
//! Rust out-of-tree sample, with C code inclusion

use kernel::prelude::*;

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
        unsafe { initialize_kprobe() };

        Ok(RustKprobes)
    }
}

impl Drop for RustKprobes {
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
