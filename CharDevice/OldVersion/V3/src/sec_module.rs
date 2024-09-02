use kernel::prelude::*;
use core::mem::MaybeUninit;
use kernel::uaccess::*;
use kernel::ioctl::*;
use kernel::sync::{new_mutex, Mutex};
use core::ptr::{addr_of_mut};

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

//--------------- STRUCTURE DEFINITION---------------

#[derive(Debug)]
struct Rule {
    rule: Vec<u8>,
}

impl Rule {
    // Takes in input a string and initialize the rule
    fn new(rule_data: Vec<u8>) -> Result<Self, Error>{
        //TODO: Validate the input 
        if rule_data.len() > RULE_SIZE{
            return Err(EINVAL);
        }
        Ok (Self { rule: rule_data,
        })
    }
    // Manually implement cloning by creating a new Vec<u8> with the same contents
    fn clone(&self) -> Result<Self, Error> {
        let mut new_vec = Vec::with_capacity(self.rule.len(),GFP_KERNEL).expect("Rule clone failed");

        // Manually copy elements from the original vector to the new vector
        for &byte in self.rule.iter() {
            if let Err(e) = new_vec.push(byte, GFP_KERNEL) {
                pr_err!("Failed to push byte to Vec: {:?}\n", e);
                return Err(e.into()); 
            }
        }

        Ok(Rule { rule: new_vec})
    }
}

#[derive(Debug)]
struct UserRule {
    uid: u32,
    rules: Vec<Rule>,
}

impl UserRule {
    // Manually implement cloning by cloning each Rule in the vector
    fn clone(&self) -> Result<Self,Error> {
        // Create a new Vec<Rule> with the same capacity as the original
        let mut cloned_rules = Vec::with_capacity(self.rules.len(),GFP_KERNEL).expect("User Rule clone failed");

        // Manually iterate and clone each rule, pushing it into the new Vec
        for rule in &self.rules {
            cloned_rules.push(rule.clone()?,GFP_KERNEL)?;
        }

        Ok(UserRule {
            uid: self.uid,
            rules: cloned_rules,
        })
    }
}


#[pin_data]
struct UserRuleStore {
    #[pin]
    store: Mutex<Vec<UserRule>>,
}

impl UserRuleStore {
    fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            store <- new_mutex!(Vec::new()),
        })
    }

    fn add_rule(&self, uid: u32, new_rule: Vec<u8>) -> Result<(), Error> {
        let mut store = self.store.lock();

        // Find the user with the given UID
        if let Some(user_rule) = store.iter_mut().find(|ur| ur.uid == uid) {
            user_rule.rules.push(Rule { rule: new_rule }, GFP_KERNEL)?;
        } else {
          // User does not exist, so create a new UserRule with the provided rule
          // Create an empty vector
          let mut rules = Vec::new();  
          // Add the new rule to the vector
          rules.push(Rule { rule: new_rule }, GFP_KERNEL)?;  

          store.push(UserRule {
              uid,
              rules,
          }, GFP_KERNEL)?;
        }

        Ok(())
    }

    fn remove_rule(&self, uid: u32, rule_to_remove: Vec<u8>) -> Result<(), Error> {
        let mut store = self.store.lock();

        if let Some(user_rule) = store.iter_mut().find(|ur| ur.uid == uid) {
            user_rule.rules.retain(|r| r.rule != rule_to_remove);

            // Remove the user if there are no more rules
            if user_rule.rules.is_empty() {
                store.retain(|ur| ur.uid != uid);
            }
        }

        Ok(())
    }

    /// Retrieves the rules associated with a specific user ID.
    // Option is used to return None if the user doesn't exists.
    fn get_rules_by_id(&self, uid: u32) -> Result<Option<Vec<Rule>>, Error> {
        let store = self.store.lock();

        if let Some(user_rule) = store.iter().find(|user_rule| user_rule.uid == uid) {
            // Create a new vector to store the cloned rules
            let mut cloned_rules = Vec::with_capacity(user_rule.rules.len(),GFP_KERNEL).expect("get_rules by id failed");

            for rule in &user_rule.rules {
                // Try to clone each rule and handle the error if it occurs
                match rule.clone() {
                    Ok(cloned_rule) => {
                        cloned_rules.push(cloned_rule, GFP_KERNEL)?;
                    }
                    Err(e) => {
                        pr_err!("Failed to clone rule: {:?}", e);
                        return Err(e);  // Propagate the error
                    }
                }
            }

            Ok(Some(cloned_rules))

        } else {
            // Return Ok(None) if the user is not found
            Ok(None)
        }
    }

    /// Retrieves all the rules in the store.
    fn get_all_rules(&self) -> Result<Vec<UserRule>, kernel::error::Error> {
        let store = self.store.lock();

        // Create a new vector to store the cloned UserRules
        let mut cloned_store = Vec::with_capacity(store.len(), GFP_KERNEL).expect("get_all_rules failed");

        for user_rule in store.iter() {
            // Try to clone each UserRule and handle the error if it occurs
            match user_rule.clone() {
                Ok(cloned_user_rule) => cloned_store.push(cloned_user_rule, GFP_KERNEL),
                Err(e) => {
                    pr_err!("Failed to clone user rule: {:?}", e);
                    return Err(e);  // Propagate the error
                }
            }?
        }

        Ok(cloned_store)
    }

}
static mut USER_RULE_STORE: Option<Pin<Box<UserRuleStore>>> = None;

//--------------- IOCTL DEFINITION ---------------
const IOCTL_MAGIC: u32 = b's' as u32; // Unique magic number

// Define IOCTL command numbers for add and remove operations
const IOCTL_ADD_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 1);
const IOCTL_REMOVE_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 2);
const IOCTL_READ_RULES: u32 = _IOR::<IoctlReadArgument>(IOCTL_MAGIC, 2);


const RULE_SIZE: usize = 256; // Max size of each rule string
const RULE_NUMBER: usize = 16;
const RULE_BUFFER_SIZE: usize = RULE_SIZE * RULE_NUMBER; // Max size of each rule string

#[repr(C)]
struct IoctlArgument {
    uid: u32,        // User ID
    rule: [u8; RULE_SIZE], // Rule string
}

#[repr(C)]
struct IoctlReadArgument {
    uid: u32,         // User ID (MAX U32 indicates no specific user ID)
    rules_buffer: [u8; RULE_BUFFER_SIZE], // Buffer to store rules
}


//--------------- IOCTL HANDLERS ---------------

#[no_mangle]
pub extern "C" fn rust_ioctl(
    _file: *mut core::ffi::c_void,
    cmd: u32,
    arg: *mut core::ffi::c_void,
) -> isize {
    if arg.is_null() {
        pr_err!("IOCTL Called without argument\n");
        return -EINVAL.to_errno() as isize;
    }

    // Safely copy the IoctlArgument from user space
    let ioctl_arg = unsafe {
        let mut buffer: [MaybeUninit<u8>; core::mem::size_of::<IoctlArgument>()] = MaybeUninit::uninit().assume_init();
        let user_ptr = arg as *const u8;
        let user_slice = UserSlice::new(user_ptr as usize, buffer.len());
        let mut reader = user_slice.reader();

        if reader.read_raw(&mut buffer).is_err() {
            pr_err!("Failed to read from user space\n");
            return -EFAULT.to_errno() as isize;
        }

        core::ptr::read(buffer.as_ptr() as *const IoctlArgument)
    };

    let uid = ioctl_arg.uid;
    // Manually construct the Vec<u8> for rule_str
    let mut rule_str = Vec::new();
    for &c in ioctl_arg.rule.iter().take_while(|&&c| c != 0) {
        if let Err(_) = rule_str.push(c, GFP_KERNEL){
            pr_err!("Failed to construct rule str");
            return -ENOMEM.to_errno() as isize;
        }
    }
    
    // Safely access the USER_RULE_STORE
    let user_rule_store = unsafe {
        // Create a mut raw pointer to RULE_STORE
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
        _ => {
            pr_err!("Unknown command\n");
            return -EINVAL.to_errno() as isize;
        }
    }

    0
}

//--------------- READ ---------------

#[no_mangle]
pub extern "C" fn rust_read(
    _file: *mut core::ffi::c_void,
    user_buffer: *mut u8,
    count: usize,
    offset: *mut u64,
) -> isize {
    // Convert the offset to usize
    let current_offset = unsafe { *offset as usize };

    // Get all rules
    // Safely access the global USER_RULE_STORE
    let rules = unsafe {
        // Safely access the global USER_RULE_STORE
        let store_ptr = addr_of_mut!(USER_RULE_STORE);
        match (*store_ptr).as_ref(){
            Some(store) => {
                match store.get_all_rules() {
                    Ok(rules) => rules, // Borrow the rules list
                    Err(e) => {
                        pr_err!("Failed to get all rules: {:?}", e);
                        return -EFAULT.to_errno() as isize;
                    }
                }
            }
            None => {
                pr_err!("User rule store is not initialized.");
                return -EFAULT.to_errno() as isize;
            }
        }
    };

    // Generate the output dynamically using Vec<u8>
    let mut output = Vec::new();
    for user_rule in rules {
        // Manually append the UID line
        if let Err(e) = output.extend_from_slice(b"---- UID: ",GFP_KERNEL){
            pr_err!("Failed to construct output str {:?}\n",e);
            return -ENOMEM.to_errno() as isize;
        }

        if let Err(e) = append_u32_to_vec(&mut output, user_rule.uid){
            pr_err!("Failed to append u32 to output str {:?}\n",e);
            return -ENOMEM.to_errno() as isize;
        }

        if let Err(e) = output.extend_from_slice(b" ----\n",GFP_KERNEL){
            pr_err!("Failed to construct output str {:?}\n",e);
            return -ENOMEM.to_errno() as isize;
        }

        // Append each rule
        for (i, rule) in user_rule.rules.iter().enumerate() {
            if let Err(e) = append_u32_to_vec(&mut output, (i + 1) as u32){
                pr_err!("Failed to append u32 to output str {:?}\n",e);
                return -ENOMEM.to_errno() as isize;
            }
            
            // Maybe this can be a PROBLEM!
            if let Err(e) = output.extend_from_slice(&rule.rule,GFP_KERNEL){
                pr_err!("Failed to construct output str {:?}\n",e);
                return -ENOMEM.to_errno() as isize;
            }
            if let Err(e) = output.extend_from_slice(b"\n",GFP_KERNEL){
                pr_err!("Failed to construct output str {:?}\n",e);
                return -ENOMEM.to_errno() as isize;
            }
        }
        if let Err(e) = output.extend_from_slice(b" ---- ---- ----\n",GFP_KERNEL){
            pr_err!("Failed to construct output str {:?}\n",e);
            return -ENOMEM.to_errno() as isize;
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

fn append_u32_to_vec(buffer: &mut Vec<u8>, value: u32) -> Result<(), Error>{
    let mut num = value;
    let mut digits = [0u8; 10]; // u32 max value is 4294967295, which is 10 digits
    let mut i = digits.len();

    // Convert the number to ASCII digits in reverse order
    loop {
        i -= 1;
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    // Append the digits to the buffer
    if let Err(e) = buffer.extend_from_slice(&digits[i..], GFP_KERNEL){
        pr_err!("Failed to convert digits to char {:?}\n",e);
        return Err(ENOMEM);
    }
    Ok(())
}

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
    let mut string = Vec::new();
    if let Err(e) = string.extend_from_slice(b"Hello Rust :)",GFP_KERNEL){
        pr_err!("Failed to construct output str {:?}\n",e);
        return;
    }

    let initial_rule = Rule::new(string).expect("Problem with rule creation");
  
    // Safely access the USER_RULE_STORE
    let user_rule_store = unsafe {
        // Create a mut raw pointer to RULE_STORE
        let store_ptr = addr_of_mut!(USER_RULE_STORE);
        match (*store_ptr).as_ref() {
            Some(store) => store,
            None => {
                pr_err!("USER_RULE_STORE not initialized\n");
                return ;
            }
        }
    };
    if let Err(e) = user_rule_store.add_rule(initial_uid, initial_rule.clone().expect("Problem with rule cloning").rule) {
        pr_err!("Failed to add initial rule: {:?}\n", e);
        return;
    }   
    pr_info!("Initialized rules with default rule for user_id {}: {:?}\n", initial_uid, initial_rule);
}
