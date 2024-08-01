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
        pr_info!("Hello, kprobes example module loaded!");

        let kprobe = Kprobe::new(
            b"do_sys_open", // This is a function name in the kernel source
            handler
        )?;
        kprobe.attach()?;

        pr_info!("Kprobe attached!");

        Ok(KprobeExample)
    }
}

impl Drop for KprobeExample {
    fn drop(&mut self) {
        pr_info!("Kprobes example module unloaded!");
    }
}

fn handler(_regs: &Registers) -> KprobeAction {
    pr_info!("kprobe handler intercepted the syscall!");
    KprobeAction::Continue
}
