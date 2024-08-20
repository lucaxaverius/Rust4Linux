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

static DEVICE_BUFFER: &[u8] = b"Hello from Rust!\n";
static BUFFER_SIZE: AtomicUsize = AtomicUsize::new(DEVICE_BUFFER.len());

#[no_mangle]
pub extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    user_buffer: *mut u8,
    count: usize,
    offset: *mut u64,
) -> isize {
    // Convert the current offset to usize
    let current_offset = unsafe { *offset as usize };
    //pr_info!("This should be the buffer current_offeset {}",current_offset);

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
    let data = &DEVICE_BUFFER[current_offset..current_offset + len];
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
    let mut buffer = Vec::new();
    // Convert raw pointer to UserPtr (alias of usize)
    let user_ptr = user_buffer as usize;
    let user_slice = UserSlice::new(user_ptr, count);

    // Use UserSliceReader to read from user and write into kernel
    let reader = user_slice.reader(); 
    
    match reader.read_all(&mut buffer, GFP_KERNEL) {
        Ok(_) => {
            // Process the received data as needed
            pr_info!("Received data length: {}\n", buffer.len());
            buffer.len() as isize
        }
        Err(e) => {
            pr_err!("Failed to read from user buffer: {:?}\n",e);
            -EFAULT.to_errno() as isize
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
}
