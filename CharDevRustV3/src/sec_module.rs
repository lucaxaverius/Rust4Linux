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
const MAX_USERS: usize = 1024;
const MAX_RULES_PER_USER: usize = 10;
const RULE_SIZE: usize = 256; // Max size of each rule string

#[derive(Clone)]
struct Rule {
    rule: Vec<u8>,
}

#[derive(Clone)]
struct UserRule {
    uid: u32,
    rules: Vec<Rule>,
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
            user_rule.rules.push(Rule { rule: new_rule }, GFP_KERNEL);
        } else {
          // User does not exist, so create a new UserRule with the provided rule
          // Create an empty vector
          let mut rules = Vec::new();  
          // Add the new rule to the vector
          rules.push(Rule { rule: new_rule }, GFP_KERNEL);  

          store.push(UserRule {
              uid,
              rules,
          }, GFP_KERNEL);
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

    fn get_rules(&self, uid: u32) -> Option<Vec<Rule>> {
        let store = self.store.lock();
        store
            .iter()
            .find(|ur| ur.uid == uid)
            .map(|user_rule| user_rule.rules.clone())
    }
}
static mut USER_RULE_STORE: Option<Pin<Box<UserRuleStore>>> = None;

//--------------- IOCTL DEFINITION ---------------
const IOCTL_MAGIC: u32 = b's' as u32; // Unique magic number
// Define IOCTL command numbers for add and remove operations
const IOCTL_ADD_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 1);
const IOCTL_REMOVE_RULE: u32 = _IOW::<IoctlArgument>(IOCTL_MAGIC, 2);

#[repr(C)]
struct IoctlArgument {
    uid: u32,        // User ID
    rule: [u8; 256], // Rule string
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
    let rule_str = ioctl_arg
        .rule
        .iter()
        .take_while(|&&c| c != 0)
        .cloned()
        .collect::<Vec<u8>>();

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
