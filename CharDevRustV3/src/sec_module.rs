use kernel::prelude::*;
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

//--------------- IOCTL DEFINITION ---------------
const IOCTL_MAGIC: u32 = b's' as u32; // Unique magic number
// Define IOCTL command numbers for add and remove operations
const IOCTL_ADD_RULE: u32 = _IOW::<IoctlCommand>(IOCTL_MAGIC, 1);
const IOCTL_REMOVE_RULE: u32 = _IOW::<IoctlCommand>(IOCTL_MAGIC, 2);

#[repr(C)]
#[derive(Debug)]
pub struct IoctlCommand {
    command: [u8; 4],  // e.g., "add\0", "rmv\0"
    rule: [u8; RULE_SIZE],   // The rule string, up to 256 bytes
}

impl IoctlCommand {
    pub fn command_str(&self) -> &str {
        core::str::from_utf8(&self.command).unwrap_or("")
    }

    pub fn rule_str(&self) -> &str {
        core::str::from_utf8(&self.rule).unwrap_or("")
    }
}

//--------------- RULES DEFINITION---------------
const MAX_USERS: usize = 1024;
const MAX_RULES_PER_USER: usize = 10;
const RULE_SIZE: usize = 256; // Max size of each rule string

#[derive(Default)]
struct UserRules {
    rules: [Option<Vec<u8>>; MAX_RULES_PER_USER],
}

impl UserRules {
    fn add_rule(&mut self, rule: Vec<u8>) -> Result<(), ()> {
        for slot in &mut self.rules {
            if slot.is_none() {
                *slot = Some(rule);
                return Ok(());
            }
        }
        Err(())
    }

    fn remove_rule(&mut self, rule: &[u8]) {
        self.rules.iter_mut().for_each(|r| {
            if let Some(existing_rule) = r {
                if existing_rule == rule {
                    *r = None;
                }
            }
        });
    }

    fn get_rules(&self) -> Vec<Vec<u8>> {
        self.rules.iter().filter_map(|r| {
            core::prelude::v1::Some(r.clone())
        }).collect()
    }
}


//--------------- RULES BUFFER DEFINITION ---------------

struct UserRuleStore {
    store: [UserRules; MAX_USERS],
}

impl UserRuleStore {
    fn new() -> Self {
        UserRuleStore {
            store: Default::default(),
        }
    }

    fn add_rule(&mut self, user_id: usize, rule: Vec<u8>) -> Result<(), ()> {
        if user_id < MAX_USERS {
            self.store[user_id].add_rule(rule)
        } else {
            Err(())
        }
    }

    fn remove_rule(&mut self, user_id: usize, rule: &[u8]) {
        if user_id < MAX_USERS {
            self.store[user_id].remove_rule(rule);
        }
    }

    fn get_rules(&self, user_id: usize) -> Vec<Vec<u8>> {
        if user_id < MAX_USERS {
            self.store[user_id].get_rules()
        } else {
            Vec::new()
        }
    }
}

static mut USER_RULE_STORE: UserRuleStore = UserRuleStore::new();

#[no_mangle]
pub extern "C" fn rust_ioctl(
    _file: *mut core::ffi::c_void,
    cmd: u32,
    arg: *mut core::ffi::c_void,
) -> isize {
    match cmd {
        IOCTL_ADD_RULE | IOCTL_REMOVE_RULE => {
            if !arg.is_null() {
                const IOCTL_COMMAND_SIZE: usize = size_of::<IoctlCommand>();

                let user_slice = UserSlice::new(arg as usize, IOCTL_COMMAND_SIZE);
                let mut reader = user_slice.reader();
                
                let mut buffer = [MaybeUninit::uninit(); IOCTL_COMMAND_SIZE];
                
                if let Err(e) = reader.read_raw(&mut buffer) {
                    pr_err!("Failed to read from user space: {:?}\n", e);
                    return -EFAULT.to_errno() as isize;
                }
                
                let ioctl_command: IoctlCommand = unsafe { core::ptr::read(buffer.as_ptr() as *const IoctlCommand) };

                match cmd {
                    IOCTL_ADD_RULE => handle_add_rule(&ioctl_command),
                    IOCTL_REMOVE_RULE => handle_remove_rule(&ioctl_command),
                    _ => -EINVAL.to_errno() as isize,
                }
            } else {
                -EINVAL.to_errno() as isize
            }
        }
        _ => -EINVAL.to_errno() as isize,
    }
}

fn handle_add_rule(command: &IoctlCommand) -> isize {
    let user_id = 0; // Placeholder for user_id, replace with actual logic to determine user_id
    let rule = command.rule_str().to_string();

    unsafe {
        if let Err(_) = USER_RULE_STORE.add_rule(user_id as usize, rule.into_bytes()) {
            return -ENOMEM.to_errno() as isize;
        }
    }

    pr_info!("Added rule for user_id {}: {}\n", user_id, rule);
    0
}

fn handle_remove_rule(command: &IoctlCommand) -> isize {
    let user_id = 0; // Placeholder for user_id, replace with actual logic to determine user_id
    let rule = command.rule_str();

    unsafe {
        USER_RULE_STORE.remove_rule(user_id as usize, rule.as_bytes());
    }

    pr_info!("Removed rule for user_id {}: {}\n", user_id, rule);
    0
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
    const USER_ID: usize = 1001;
    const INITIAL_RULE: &str = "Hello Rust";

    let rule_bytes = INITIAL_RULE.as_bytes().to_vec();

    unsafe {
        if let Err(_) = USER_RULE_STORE.add_rule(USER_ID, rule_bytes) {
            pr_err!("Failed to add initial rule to USER_RULE_STORE\n");
        }
    }

    pr_info!("Initialized rules with default rule for user_id {}: {}\n", USER_ID, INITIAL_RULE);
}
