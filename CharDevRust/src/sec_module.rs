use kernel::prelude::*;
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel::uaccess::*;

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

static DEVICE_BUFFER: &[u8] = b"Hello from Rust!";
static BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEVICE_BUFFER.len());

#[no_mangle]
pub extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    user_buffer: *mut u8,
    count: usize,
    offset: *mut u64,
) -> isize {
    let current_offset = unsafe { *offset as usize };
    let len = core::cmp::min(count, BUFFER_SIZE.load(Ordering::Relaxed) - current_offset);
    let data = &DEVICE_BUFFER[current_offset..current_offset + len];

    if len > 0 {
        // Convert raw pointer to UserPtr (alias of usize)
        let user_ptr = unsafe{ *user_buffer as usize};
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
            Err(_) => -EFAULT.to_errno() as isize, // Convert Error to isize
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
    let mut buffer = Vec::new();
    // Convert raw pointer to UserPtr (alias of usize)
    let user_ptr = unsafe{ *user_buffer as usize};
    let user_slice = UserSlice::new(user_ptr, count);

    // Use UserSliceReader to read from user and write into kernel
    let reader = user_slice.reader(); 
    
    match reader.read_all(&mut buffer, GFP_KERNEL) {
        Ok(_) => {
            // Process the received data as needed
            pr_info!("Received data length: {}\n", buffer.len());
            buffer.len() as isize
        }
        Err(_) => -EFAULT.to_errno() as isize,
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
}
