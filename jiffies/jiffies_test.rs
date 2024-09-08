// SPDX-License-Identifier: GPL-2.0

//! Jiffies bindings test

use kernel::{jiffies, prelude::*};

module! {
    type: JiffiesTest,
    name: "jiffies_test",
    author: "Luca Saverio Esposito",
    description: "Testing the bindings to jiffies functions",
    license: "GPL v2",
}

struct JiffiesTest;

impl kernel::Module for JiffiesTest {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        let jiffies = 1000;
        let msecs = jiffies::jiffies_to_msecs(jiffies);
        pr_info!("Milliseconds: {}\n", msecs);
        let jiffies = 1001;
        let usecs = jiffies::jiffies_to_usecs(jiffies);
        pr_info!("Microseconds: {}\n", usecs);
        Ok(JiffiesTest)
    }
}
