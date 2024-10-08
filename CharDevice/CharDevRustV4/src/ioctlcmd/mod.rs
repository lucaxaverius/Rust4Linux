// ioctl.rs
//--------------- IOCTL DEFINITION ---------------
// This file contains everything about the IOCTL: constant, structure and the various handlers.

use kernel::uaccess::*;
use kernel::ioctl::*;
use core::ptr::{addr_of_mut};
use kernel::{str::CString, fmt};
use kernel::prelude::*;
use core::mem::MaybeUninit;

pub(crate) mod structures;
use crate::ioctlcmd::structures::constant::{RULE_SIZE,RULE_BUFFER_SIZE};
use crate::ioctlcmd::structures::{UserRuleStore,UserRule};

// Declare the external variable
extern "Rust" {
    pub(crate) static mut USER_RULE_STORE: Option<Pin<Box<UserRuleStore>>>;
}

const IOCTL_MAGIC: u32 = b's' as u32; // Unique magic number

// Define IOCTL command numbers for add and remove operations
const IOCTL_ADD_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 1);
const IOCTL_REMOVE_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 2);
const IOCTL_READ_RULES: u32 = _IOR::<IoctlReadArgument>(IOCTL_MAGIC, 3);


#[repr(C)]
struct IoctlArgument {
    uid: u32,        // User ID
    rule: [u8; RULE_SIZE], // Rule string as byte array
}

// Methods for IoctlArgument
impl IoctlArgument {    
    // Helper function to create a CString from the rule field in IoctlArgument
    fn create_cstring_from_rule(&self) -> Result<CString, Error> {
        // Find the length of the rule by identifying the first NUL byte
        let rule_len = self.rule.iter().position(|&byte| byte == 0).unwrap_or(RULE_SIZE);

        //pr_info!("The corresponding rule len is: {}",rule_len);
            
       // Allocate a new vector with enough space for the rule and a null terminator
       let mut rule_with_null = Vec::with_capacity(rule_len + 1, GFP_KERNEL).expect("Impossible to alloc vector");

       // Copy the original rule data into the vector
       for &byte in &self.rule[..rule_len] {
           if let Err(e) = rule_with_null.push(byte, GFP_KERNEL){
                pr_err!("Failed during rule copy: {:?}\n", e);
                return Err(ENOMEM);
            }
       }

       // Ensure the vector is null-terminated
       if let Err(e) = rule_with_null.push(0, GFP_KERNEL){
            pr_err!("Failed to append null byte {:?}\n", e);
            return Err(ENOMEM);
       }

        // Attempt to create a CStr from the bytes array
        let cstr = match CStr::from_bytes_with_nul(&rule_with_null) {
            Ok(cstr) => cstr,
            Err(e) => {
                pr_err!("Failed to create CStr from bytes: {:?} \nThe error is: {:?}", rule_with_null,e);
                return Err(EINVAL); // Return EINVAL on error
            }
        };

        // Convert CStr to &str; handle UTF-8 validation
        let rule_str = match cstr.to_str() {
            Ok(s) => s,
            Err(e) => {
                pr_err!("Failed to convert CStr to str: {:?}", e);
                return Err(EINVAL); // Return EINVAL on UTF-8 error
            }
        };

        // Create CString using try_from_fmt with formatting
        match CString::try_from_fmt(fmt!("{}", rule_str)) {
            Ok(cstring) => Ok(cstring),
            Err(e) => {
                pr_err!("Failed to create CString using try_from_fmt: {:?}", e);
                Err(EINVAL) // Return EINVAL on CString creation error
            }
        }
    }
}

#[repr(C)]
struct IoctlReadArgument {
    uid: u32,         // User ID (MAX U32 indicates no specific user ID)
    rules_buffer: [u8; RULE_BUFFER_SIZE], // Buffer to store rules
}

//--------------- IOCTL HANDLERS ---------------

/// Handles IOCTL commands for managing user-defined security rules.
///
/// This function processes IOCTL commands from user space to add, remove, 
/// or read security rules associated with specific user IDs. It supports 
/// the following commands:
/// - `IOCTL_ADD_RULE`: Adds a rule for a given user ID.
/// - `IOCTL_REMOVE_RULE`: Removes a rule for a given user ID.
/// - `IOCTL_READ_RULES`: Retrieves rules for a specific user ID.
///
/// # Parameters
///
/// - `_file`: A pointer to a file object (unused).
/// - `cmd`: The IOCTL command to execute (`u32`).
/// - `arg`: A pointer to the argument structure passed from user space.
///
/// # Returns
///
/// - `0` on success.
/// - A negative error code on failure.
///
/// # Errors
///
/// This function returns various error codes depending on the failure conditions:
/// - `-EINVAL` if an invalid argument or command is provided.
/// - `-EFAULT` if reading or writing from/to user space fails.
/// - `-ENOMEM` if there is a memory allocation error.
///
/// # Safety
///
/// This function involves unsafe operations such as reading directly from user space.
/// It must ensure that all pointer dereferences and memory accesses are valid.
///
/// # Examples
///
/// ```rust
/// // Example of calling `rust_ioctl` to add a rule
/// rust_ioctl(file_ptr, IOCTL_ADD_RULE, ioctl_arg_ptr);
/// ```
/// 
#[no_mangle]
pub(crate) extern "C" fn rust_ioctl(
    _file: *mut core::ffi::c_void,
    cmd: u32,
    arg: *mut core::ffi::c_void,
) -> isize {
    if arg.is_null() {
        pr_err!("IOCTL Called without argument\n");
        return -EINVAL.to_errno() as isize;
    }

    match cmd {
        IOCTL_ADD_RULE | IOCTL_REMOVE_RULE => {
            // Safely copy the IoctlArgument from user space
            let ioctl_arg = unsafe {
                let mut buffer: [MaybeUninit<u8>; core::mem::size_of::<IoctlArgument>()] = MaybeUninit::uninit().assume_init();
                let user_ptr = arg as *const u8;
                let user_slice = UserSlice::new(user_ptr as usize, buffer.len());
                let mut reader = user_slice.reader();

                if reader.read_raw(&mut buffer).is_err() {
                    pr_err!("Failed to read from user space for ADD/REMOVE IOCTL\n");
                    return -EFAULT.to_errno() as isize;
                }

                core::ptr::read(buffer.as_ptr() as *const IoctlArgument)
            };

            let uid = ioctl_arg.uid;
            let rule_str = match ioctl_arg.create_cstring_from_rule() {
                Ok(cstring) => cstring,
                Err(e) => {
                    pr_err!("Failed to construct rule string: {:?}\n", e);
                    return -EINVAL.to_errno() as isize;
                }
            };

            // Safely access the USER_RULE_STORE
            let user_rule_store = unsafe {
                let store_ptr = addr_of_mut!(USER_RULE_STORE);
                match (*store_ptr).as_ref() {
                    Some(store) => store,
                    None => {
                        pr_err!("USER_RULE_STORE not initialized\n");
                        return -EINVAL.to_errno() as isize;
                    }
                }
            };

            match cmd {
                IOCTL_ADD_RULE => {
                    if let Err(e) = user_rule_store.add_rule(uid, rule_str) {
                        pr_err!("Failed to add rule: {:?}\n", e);
                        return -EFAULT.to_errno() as isize;
                    }
                }
                IOCTL_REMOVE_RULE => {
                    if let Err(e) = user_rule_store.remove_rule(uid, rule_str) {
                        pr_err!("Failed to remove rule: {:?}\n", e);
                        return -EFAULT.to_errno() as isize;
                    }
                }
                _ => unreachable!(),
            }
        }

        IOCTL_READ_RULES => {
            // Safely copy the IoctlReadArgument from user space
            let mut ioctl_read_arg = unsafe {
                let mut buffer: [MaybeUninit<u8>; core::mem::size_of::<IoctlReadArgument>()] = MaybeUninit::uninit().assume_init();
                let user_ptr = arg as *const u8;
                let user_slice = UserSlice::new(user_ptr as usize, buffer.len());
                let mut reader = user_slice.reader();

                if reader.read_raw(&mut buffer).is_err() {
                    pr_err!("Failed to read from user space for READ IOCTL\n");
                    return -EFAULT.to_errno() as isize;
                }

                core::ptr::read(buffer.as_ptr() as *const IoctlReadArgument)
            };

            // Access the USER_RULE_STORE and gets rules by UID
            let rules = unsafe {
                let store_ptr = addr_of_mut!(USER_RULE_STORE);
                match (*store_ptr).as_ref() {
                    Some(store) => match store.get_rules_by_id(ioctl_read_arg.uid) {
                        Ok(rules) => rules, // Borrow the rules list
                        Err(e) => {
                            pr_err!("Failed to get rules for UID: {}\n{:?}",ioctl_read_arg.uid, e);
                            return -EINVAL.to_errno() as isize;
                        }
                    },
                    None => {
                        pr_err!("USER_RULE_STORE not initialized\n");
                        return -EINVAL.to_errno() as isize;
                    }
                }
            };

            let mut output : Vec<u8> = Vec::new();
            if let Some(user_rule) = rules {
                if let Err(e) = pretty_print_rules(user_rule, &mut output){
                    return e.to_errno() as isize;
                }
            }
            
            // Calculate the length of the data to be copied
            let output_len = core::cmp::min(output.len(), ioctl_read_arg.rules_buffer.len());

            // Update the local kernel copy's buffer with the output
            ioctl_read_arg.rules_buffer[..output_len].copy_from_slice(&output[..output_len]);

            // Write the updated buffer back to user space
            // Moves u32 size forward to match the addr of the buffer inside the IoctReadArgument.
            let user_buffer_ptr = unsafe { (arg as *mut u8).add(core::mem::size_of::<u32>()) as usize }; 
            
            let user_slice = UserSlice::new(user_buffer_ptr, output_len);
            let mut writer = user_slice.writer();

            // Write the kernel buffer to the user's buffer
            if let Err(e) = writer.write_slice(&ioctl_read_arg.rules_buffer[..output_len]) {
                pr_err!("Failed to write back to user space for READ IOCTL: {:?}\n", e);
                return -EFAULT.to_errno() as isize;
            }
        }

        _ => {
            pr_err!("Unknown IOCTL command\n");
            return -EINVAL.to_errno() as isize;
        }
    }

    0
}

//--------------- READ ---------------

#[no_mangle]
pub(crate) extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    user_buffer: *mut u8,
    count: usize,
    offset: *mut u64,
) -> isize {
    // Convert the offset to usize
    let current_offset = unsafe { *offset as usize };

    // Safely access the global USER_RULE_STORE
    let rules = unsafe {
        let store_ptr = addr_of_mut!(USER_RULE_STORE);
        match (*store_ptr).as_ref() {
            Some(store) => match store.get_all_rules() {
                Ok(rules) => rules, // Borrow the rules list
                Err(e) => {
                    pr_err!("Failed to get all rules: {:?}", e);
                    return -EFAULT.to_errno() as isize;
                }
            },
            None => {
                pr_err!("User rule store is not initialized.");
                return -EFAULT.to_errno() as isize;
            }
        }
    };

    // Generate the output dynamically using Vec<u8>
    let mut output : Vec<u8> = Vec::new();

    for user_rule in rules {
        if let Err(e) = pretty_print_rules(user_rule, &mut output){
            return e.to_errno() as isize;
        }    
    }

    // Check if the offset is beyond the buffer
    if current_offset >= output.len() {
        return 0; // EOF
    }

    // Determine the number of bytes to read
    let len = core::cmp::min(count, output.len() - current_offset);

    // Data slice to copy to the user buffer
    let data = &output[current_offset..current_offset + len];

    if len > 0 {
        // Convert raw pointer to UserPtr (alias of usize)
        let user_ptr = user_buffer as usize;

        let user_slice = UserSlice::new(user_ptr, len);
        // Use UserSliceWriter to write data from device to userspace
        let mut writer = user_slice.writer();

        match writer.write_slice(data) {
            Ok(_) => {
                // Update the offset in the caller's memory
                let new_offset = current_offset + len;
                unsafe { *offset = new_offset as u64 };
                len as isize
            }
            Err(e) => {
                pr_err!("Failed to write to user buffer: {:?}\n", e);
                -EFAULT.to_errno() as isize
            }
        }
    } else {
        0 // EOF
    }
}

fn pretty_print_rules(rules: UserRule, output: &mut Vec<u8>)-> Result<(),Error>{
    // Append the UID line using CString::try_from_fmt
    let uid_str = match CString::try_from_fmt(format_args!("---- UID: {} ----\n", rules.uid)) {
        Ok(cstring) => cstring,
        Err(e) => {
            pr_err!("Failed to construct UID string: {:?}\n", e);
            return Err(ENOMEM);
        }
    };
    if let Err(e) = output.extend_from_slice(uid_str.as_bytes(), GFP_KERNEL) {
        pr_err!("Failed to append UID string: {:?}\n", e);
        return Err(ENOMEM);
    }

    // Append each rule
    for (i, rule) in rules.rules.iter().enumerate() {
        
        // Append the rule number using CString::try_from_fmt
        let rule_num_str = match CString::try_from_fmt(format_args!("Rule {}: ", i + 1)) {
            Ok(cstring) => cstring,
            Err(e) => {
                pr_err!("Failed to construct rule number string: {:?}\n", e);
                return Err(ENOMEM);
            }
        };
        if let Err(e) = output.extend_from_slice(rule_num_str.as_bytes(), GFP_KERNEL) {
            pr_err!("Failed to append rule number string: {:?}\n", e);
            return Err(ENOMEM);
        }

        //pr_info!("The rule string is: {}",rule.rule.to_str().expect("Can't display the string"));

        // Append the rule itself
        if let Err(e) = output.extend_from_slice(rule.rule.as_bytes(), GFP_KERNEL) {
            pr_err!("Failed to append rule string: {:?}\n", e);
            return Err(ENOMEM);
        }
        
        // Append a newline
        if let Err(e) = output.extend_from_slice(b"\n", GFP_KERNEL) {
            pr_err!("Failed to append newline: {:?}\n", e);
            return Err(ENOMEM);
        }
    }

    // Append the footer line
    let footer_str = match CString::try_from_fmt(format_args!(" ---- ---- ----\n")) {
        Ok(cstring) => cstring,
        Err(e) => {
            pr_err!("Failed to construct footer string: {:?}\n", e);
            return Err(ENOMEM);
        }
    };
    if let Err(e) = output.extend_from_slice(footer_str.as_bytes(), GFP_KERNEL) {
        pr_err!("Failed to append footer string: {:?}\n", e);
        return Err(ENOMEM);
    }
    Ok(())
}