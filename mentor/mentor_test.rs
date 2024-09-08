// SPDX-License-Identifier: GPL-2.0

//! Mentor test

use kernel::{mentor, prelude::*};

module! {
    type: MentorTest,
    name: "mentor_test",
    author: "Rust for Linux Contributors",
    description: "Mentor Test",
    license: "GPL v2",
}

struct MentorTest;

impl kernel::Module for MentorTest {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        // Read module parameters
        let addr = 0;

        // Never use bindings directly! Always create a safe abstraction.

        // The proper way.
        pr_info!("--- With a safe abstraction\n");


        pr_info!("Reading from address {}\n", addr);
        let value = mentor::read(addr)?;
        pr_info!("Read value = {}\n", value);

        //let value = 42;

        //pr_info!("Writing value {} to address {}\n", value, addr);
        // mentor::write(addr, value)?;

        pr_info!("Reading from address {}\n", addr);
        let value = mentor::read(addr)?;
        pr_info!("Read value = {}\n", value);

        pr_info!("Reading from address {}\n", addr+1);
        let value = mentor::read(addr+1)?;
        pr_info!("Read value = {}\n", value);

        //let value = 69;
        //pr_info!("Writing value {} to address {}\n", value, addr+1);
        //mentor::write(addr, value)?;
        
        pr_info!("Reading from address {}\n", addr+2);
        let value = mentor::read(addr+2)?;
        pr_info!("Read value = {}\n", value);

        let total_writes = mentor::read_total_writes();
        pr_info!("Total writes = {}\n", total_writes);

        // Whatever we try to do here, as long as it is safe code,
        // we cannot produce UB.
        let bad_addr = 0x42;
        pr_info!("Reading from address {}\n", bad_addr);
        if mentor::read(bad_addr).is_err() {
            pr_info!("Expected failure\n");
        }

        pr_info!("Writing to address {}\n", bad_addr);
        if mentor::write(bad_addr, 69).is_err() {
            pr_info!("Expected failure\n");
        }

        Ok(MentorTest)
    }
}
