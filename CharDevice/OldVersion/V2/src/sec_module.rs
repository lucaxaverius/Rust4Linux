use kernel::prelude::*;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::mem::{MaybeUninit, size_of};
use kernel::uaccess::*;
use kernel::ioctl::*;

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

static mut DEVICE_BUFFER: Vec<u8> = Vec::new();
static BUFFER_SIZE: AtomicUsize = AtomicUsize::new(0);

const IOCTL_MAGIC: u32 = b's' as u32; // Unique magic number
// Define IOCTL command numbers for add and remove operations
const IOCTL_ADD_RULE: u32 = _IOW::<IoctlCommand>(IOCTL_MAGIC, 1);
const IOCTL_REMOVE_RULE: u32 = _IOW::<IoctlCommand>(IOCTL_MAGIC, 2);

#[repr(C)]
struct IoctlCommand {
    command: [u8; 4],  // e.g., "add\0", "rmv\0" (4 bytes, including the null terminator)
    rule: [u8; 256],   // The rule string, up to 256 bytes
}


#[no_mangle]
pub extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    user_buffer: *mut u8,
    count: usize,
    offset: *mut u64,
) -> isize {
    // Convert the offset to usize
    let current_offset = unsafe { *offset as usize };
    //pr_info!("This should be the buffer current_offset {}",current_offset);

    let buffer_size = BUFFER_SIZE.load(Ordering::Relaxed);
    //pr_info!("This should be the buffer size {}",buffer_size);

    // Check if the offset is beyond the buffer
    if current_offset >= buffer_size {
        return 0; // EOF
    }

    // Determine the number of bytes to read
    let len = core::cmp::min(count, buffer_size - current_offset);
    //pr_info!("This should be bytes to read {}",len);

    // Data slice to copy to the user buffer
    let data = unsafe {&DEVICE_BUFFER[current_offset..current_offset + len]};
    //pr_info!("This should be the buffer DATA slice {:?}",data);

    if len > 0 {
        // Convert raw pointer to UserPtr (alias of usize)
        let user_ptr = user_buffer as usize;
        //pr_info!("UserPtr address: {:#x}", user_ptr);
        
        let user_slice = UserSlice::new(user_ptr, len);
        // Use UserSliceWriter to write data from device to userspace
        let mut writer = user_slice.writer(); 
        
        match writer.write_slice(&data) {
            Ok(_) => {
                // Update the offset in the caller's memory
                let new_offset = current_offset + len;
                unsafe { *offset = new_offset as u64 };
                len as isize
            }
            Err(e) => {
                pr_err!("Failed to write to user buffer: {:?}\n", e);
                // Convert Error to isize
                -EFAULT.to_errno() as isize
            } 
        }
    } else {
        0 // EOF
    }
}

#[no_mangle]
pub extern "C" fn rust_write(
    _file: *mut core::ffi::c_void,
    user_buffer: *const u8,
    count: usize,
    _offset: *mut u64,
) -> isize {
    let user_ptr = user_buffer as usize;
    let user_slice = UserSlice::new(user_ptr, count);

    // Use UserSliceReader to read from user and write into kernel
    let reader = user_slice.reader(); 
    let mut buffer: Vec<u8> = Vec::with_capacity(count, GFP_KERNEL).expect("Failed to allocate buffer");

    match reader.read_all(&mut buffer, GFP_KERNEL) {
        Ok(_) => {
            unsafe {
                // Append a newline if needed
                if !buffer.ends_with(b"\n") {
                    if let Err(e) = buffer.push(b'\n', GFP_KERNEL) {
                        pr_err!("Failed to append newline: {:?}\n", e);
                        return -EFAULT.to_errno() as isize;
                    }
                }

                // Append the new data to the existing DEVICE_BUFFER
                if let Err(e) = DEVICE_BUFFER.extend_from_slice(&buffer, GFP_KERNEL) {
                    pr_err!("Failed to extend DEVICE_BUFFER: {:?}\n", e);
                    return -EFAULT.to_errno() as isize;
                }

                // Update BUFFER_SIZE to reflect the new size of DEVICE_BUFFER
                BUFFER_SIZE.store(DEVICE_BUFFER.len(), Ordering::Relaxed);
            }

            pr_info!("Appended data length: {}\n", buffer.len());
            buffer.len() as isize
        }
        Err(e) => {
            pr_err!("Failed to read from user buffer: {:?}\n", e);
            -EFAULT.to_errno() as isize
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_ioctl(
    _file: *mut core::ffi::c_void,
    cmd: u32,
    arg: u64,
) -> isize {
    // We expect a pointer to an IoctlCommand struct from user space
    let user_ptr = arg as *const u8;

    // Allocate an uninitialized buffer of the appropriate size
    let mut buffer: [MaybeUninit<u8>; size_of::<IoctlCommand>()] = unsafe {MaybeUninit::uninit().assume_init()};

    // Safely copy data from user space
    let user_slice =  UserSlice::new(user_ptr as usize, buffer.len());
    let mut reader = user_slice.reader();

    if let Err(e) = reader.read_raw(&mut buffer) {
        pr_err!("Failed to read from user space: {:?}\n", e);
        return -EFAULT.to_errno() as isize;
    }

    // Transmute the raw buffer into an IoctlCommand struct
    let ioctl_command: IoctlCommand = unsafe { core::ptr::read(buffer.as_ptr() as *const IoctlCommand) };

    // Determine the command and process accordingly
    match cmd {
        IOCTL_ADD_RULE => handle_add_rule(&ioctl_command),
        IOCTL_REMOVE_RULE => handle_remove_rule(&ioctl_command),
        _ => {
            pr_err!("Unknown IOCTL command: {}\n", cmd);
            -EINVAL.to_errno() as isize
        }
    }
}

fn handle_add_rule(cmd: &IoctlCommand) -> isize {
    let command_str = core::str::from_utf8(&cmd.command).unwrap_or("").trim_end_matches('\0');
    let rule_str = core::str::from_utf8(&cmd.rule).unwrap_or("").trim_end_matches('\0');

    if command_str == "add" {
        pr_info!("Adding rule: {}\n", rule_str);
        // Handle adding the rule to your buffer or data structure
        add_rule(rule_str)
    } else {
        pr_err!("Invalid command for adding rule: {}\n", command_str);
        -EINVAL.to_errno() as isize
    }
}

fn handle_remove_rule(cmd: &IoctlCommand) -> isize {
    let command_str = core::str::from_utf8(&cmd.command).unwrap_or("").trim_end_matches('\0');
    let rule_str = core::str::from_utf8(&cmd.rule).unwrap_or("").trim_end_matches('\0');

    if command_str == "rmv" {
        pr_info!("Removing rule: {}\n", rule_str);
        // Handle removing the rule from your buffer or data structure
        remove_rule(rule_str)
    } else {
        pr_err!("Invalid command for removing rule: {}\n", command_str);
        -EINVAL.to_errno() as isize
    }
}

fn add_rule(rule: &str) -> isize {
    unsafe {
        let rule_bytes = rule.as_bytes();

        // Create a Vec<u8> with capacity for the rule and an extra byte for the newline
        let mut rule_vec = Vec::with_capacity(rule_bytes.len() + 1, GFP_KERNEL).expect("Failed to allocate buffer");

        // Extend the Vec<u8> with the rule bytes
        if let Err(e) = rule_vec.extend_from_slice(rule_bytes, GFP_KERNEL) {
            pr_err!("Failed to extend rule_vec: {:?}\n", e);
            return -EFAULT.to_errno() as isize;
        }

        // Check if the rule already ends with a newline
        if !rule_vec.ends_with(b"\n") {
        // Append a newline
            if let Err(e) = rule_vec.push(b'\n', GFP_KERNEL) {
                pr_err!("Failed to append newline: {:?}\n", e);
                return -EFAULT.to_errno() as isize;
            }
        }
        // Append the new rule to the existing DEVICE_BUFFER
        if let Err(e) = DEVICE_BUFFER.extend_from_slice(&rule_vec, GFP_KERNEL) {
            pr_err!("Failed to extend DEVICE_BUFFER: {:?}\n", e);
            return -EFAULT.to_errno() as isize;
        }

        // Update BUFFER_SIZE to reflect the new size of DEVICE_BUFFER
        BUFFER_SIZE.store(DEVICE_BUFFER.len(), Ordering::Relaxed);
    }
    pr_info!("Rule added successfully\n");
    0
}

fn remove_rule(rule: &str) -> isize {
    unsafe {
        let rule_bytes = rule.as_bytes();
        if let Some(pos) = DEVICE_BUFFER.windows(rule_bytes.len()).position(|window| window == rule_bytes) {
            DEVICE_BUFFER.drain(pos..pos + rule_bytes.len());
            BUFFER_SIZE.store(DEVICE_BUFFER.len(), Ordering::Relaxed);
            pr_info!("Rule removed successfully\n");
            0
        } else {
            pr_err!("Rule not found\n");
            -ENOENT.to_errno() as isize
        }
    }
}


struct SecModule;

impl kernel::Module for SecModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        // Initialize your rule set
        init_rules();

        unsafe {
            if create_device() < 0 {
                return Err(EINVAL);
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

fn init_rules() {
    // Initialize rule set or other necessary data structures
    // Initialize the buffer with "Hello Rust"
    let initial_data = b"Hello Rust\n";
    unsafe {
        if let Err(e) = DEVICE_BUFFER.extend_from_slice(initial_data, GFP_KERNEL) {
            pr_err!("Failed to extend DEVICE_BUFFER: {:?}\n", e);
        }        
        BUFFER_SIZE.store(DEVICE_BUFFER.len(), Ordering::Relaxed);
    }
    pr_info!("Buffer initialized correctly");
}
