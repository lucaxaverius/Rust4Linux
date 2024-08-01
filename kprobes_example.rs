// SPDX-License-Identifier: GPL-2.0

//! Rust out-of-tree sample

#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::kprobes::*;

module! {
    type: RustOutOfTree,
    name: "Rust Kprobes for kernel hacking",
    author: "Luca Saverio Esposito",
    description: "A simple test to verify how kprobes are implemented in Rust",
    license: "GPL",
}

struct KprobeExample;

impl kernel::Module for KprobeExample {
    fn init() -> Result<Self> {
        pr_info!("Rust kprobe module loaded!\n");

        // Call the C function to initialize kprobe
        unsafe { initialize_kprobe() };

        Ok(Module)
    }
}

impl Drop for KprobeExample {
    fn drop(&mut self) {
        pr_info!("Kprobes example module unloaded!");
        
        // Call the C function to clean up kprobe
        unsafe { cleanup_kprobe() };
    }
}

fn handler(_regs: &Registers) -> KprobeAction {
    pr_info!("kprobe handler intercepted the syscall!");
    KprobeAction::Continue
}

// FFI declarations
extern "C" {
    fn initialize_kprobe();
    fn cleanup_kprobe();
}