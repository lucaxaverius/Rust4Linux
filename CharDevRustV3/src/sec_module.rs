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
    const USER_ID: usize = 1001;
    const INITIAL_RULE: &str = "Hello Rust";
    pr_info!("Initialized rules with default rule for user_id {}: {}\n", USER_ID, INITIAL_RULE);
}
