// structures.rs
//--------------- STRUCTURE DEFINITION---------------
// This file contains the structures used to manage the rules.
use kernel::{str::CString, fmt};
use kernel::prelude::*;
use kernel::sync::{new_mutex, Mutex};

mod constant;
use crate::structures::constant::RULE_SIZE;

#[derive(Debug)]
pub struct Rule {
    pub rule: CString,
}

impl Rule {
    // Takes in input a string and initialize the rule
    pub fn new(rule_data: CString) -> Result<Self, Error> {
        if rule_data.as_bytes().len() > RULE_SIZE {
            return Err(EINVAL);
        }
        Ok(Self { rule: rule_data })
    }

    // Manually implement cloning by creating a new CString with the same contents
    pub fn clone(&self) -> Result<Self, Error> {
        Ok(Rule {
            rule: CString::try_from_fmt(fmt!("{}", self.rule.to_str().expect("UTF8 error during Clone"))).unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct UserRule {
    uid: u32,
    rules: Vec<Rule>,
}

impl UserRule {
    // Manually implement cloning by cloning each Rule in the vector
    fn clone(&self) -> Result<Self, Error> {
        // Create a new Vec<Rule> with the same capacity as the original
        let mut cloned_rules = Vec::with_capacity(self.rules.len(), GFP_KERNEL).expect("User Rule clone failed");

        // Manually iterate and clone each rule, pushing it into the new Vec
        for rule in &self.rules {
            cloned_rules.push(rule.clone()?, GFP_KERNEL)?;
        }

        Ok(UserRule {
            uid: self.uid,
            rules: cloned_rules,
        })
    }
}

#[pin_data]
pub struct UserRuleStore {
    #[pin]
    store: Mutex<Vec<UserRule>>,
}

impl UserRuleStore {
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            store <- new_mutex!(Vec::new()),
        })
    }

    pub fn add_rule(&self, uid: u32, new_rule: CString) -> Result<(), Error> {
        let mut store = self.store.lock();
        // pr_info!("The rule string is: {}",new_rule.to_str().expect("Can't display the string"));

        // Find the user with the given UID
        if let Some(user_rule) = store.iter_mut().find(|ur| ur.uid == uid) {
            user_rule.rules.push(Rule { rule: new_rule }, GFP_KERNEL)?;
        } else {
            // User does not exist, so create a new UserRule with the provided rule
            let mut rules = Vec::new();
            rules.push(Rule { rule: new_rule }, GFP_KERNEL)?;

            store.push(UserRule {
                uid,
                rules,
            }, GFP_KERNEL)?;
        }

        Ok(())
    }

    fn remove_rule(&self, uid: u32, rule_to_remove: CString) -> Result<(), Error> {
        let mut store = self.store.lock();

        if let Some(user_rule) = store.iter_mut().find(|ur| ur.uid == uid) {
            user_rule.rules.retain(|r| r.rule.as_bytes() != rule_to_remove.as_bytes());

            // Remove the user if there are no more rules
            if user_rule.rules.is_empty() {
                store.retain(|ur| ur.uid != uid);
            }
        }

        Ok(())
    }

    /// Retrieves the rules associated with a specific user ID.
    fn get_rules_by_id(&self, uid: u32) -> Result<Option<Vec<Rule>>, Error> {
        let store = self.store.lock();

        if let Some(user_rule) = store.iter().find(|user_rule| user_rule.uid == uid) {
            let mut cloned_rules = Vec::with_capacity(user_rule.rules.len(), GFP_KERNEL).expect("get_rules by id failed");

            for rule in &user_rule.rules {
                match rule.clone() {
                    Ok(cloned_rule) => cloned_rules.push(cloned_rule, GFP_KERNEL)?,
                    Err(e) => {
                        pr_err!("Failed to clone rule: {:?}", e);
                        return Err(e);  // Propagate the error
                    }
                }
            }

            Ok(Some(cloned_rules))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all the rules in the store.
    fn get_all_rules(&self) -> Result<Vec<UserRule>, kernel::error::Error> {
        let store = self.store.lock();

        // Create a new vector to store the cloned UserRules
        let mut cloned_store = Vec::with_capacity(store.len(), GFP_KERNEL).expect("get_all_rules failed");

        for user_rule in store.iter() {
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


