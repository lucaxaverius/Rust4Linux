//! `sec_module`: A Rust kernel module for managing security rules.
//!
//! This crate provides the implementation of a Linux kernel module 
//! that manages user-defined security rules. It supports adding, 
//! removing, and retrieving rules for specific users via IOCTL system calls.
//!
//! # Features
//! - Add security rules for specific user IDs.
//! - Remove existing rules for specific user IDs.
//! - Retrieve all rules or rules for a specific user ID.
//!
//! # Usage
//! To check the how to use the module, check the man in sec_tool 


use kernel::prelude::*;
use kernel::{str::CString, fmt};
use core::ptr::{addr_of_mut};
mod ioctlcmd;

use crate::ioctlcmd::{rust_ioctl, rust_read};
use crate::ioctlcmd::structures::{UserRuleStore,Rule};

module! {
    type: SecModule,
    name: "sec_module",
    author: "Your Name",
    description: "A security module to register and retrieve rules",
    license: "GPL",
}

extern "C" {
    fn create_device() -> i32;
    fn remove_device();
}


#[no_mangle]
pub(crate) static mut USER_RULE_STORE: Option<Pin<Box<UserRuleStore>>> = None;

struct SecModule;

impl kernel::Module for SecModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        // Initialize the rule store
        unsafe {
            // Use of mutable static in unsafe, the initialization is done by one single thread. 
            // It will not cause any problem.
            USER_RULE_STORE = match Box::pin_init(UserRuleStore::new(), GFP_KERNEL) {
                Ok(store) => Some(store),
                Err(e) => {
                    pr_err!("Failed to initialize USER_RULE_STORE: {:?}\n", e);
                    return Err(e);
                }
            };

            if create_device() < 0 {
                return Err(EINVAL);
            }

        }
        init_rules();
        pr_info!("SecModule initialized\n");
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



fn init_rules() {
    let initial_uid: u32 = 1001;

    // Create an initial CString rule
    let string = CString::try_from_fmt(fmt!("{}","Hello Rust :)")).expect("CString creation failed");

    let initial_rule = Rule::new(string).expect("Problem with rule creation");

    // Safely access the USER_RULE_STORE
    let user_rule_store = unsafe {
        let store_ptr = addr_of_mut!(USER_RULE_STORE);
        match (*store_ptr).as_ref() {
            Some(store) => store,
            None => {
                pr_err!("USER_RULE_STORE not initialized\n");
                return;
            }
        }
    };

    if let Err(e) = user_rule_store.add_rule(initial_uid, initial_rule.clone().expect("Problem with rule cloning").rule) {
        pr_err!("Failed to add initial rule: {:?}\n", e);
        return;
    }

    pr_info!("Initialized rules with default rule for user_id {}: {:?}\n", initial_uid, initial_rule);
}
