// linked_list_test.rs

// SPDX-License-Identifier: GPL-2.0

//! Linked List Test Module

use kernel::{prelude::*, time::{self, Ktime}, linked_list::*};
use kernel::container_of;
use kernel::bindings;

module! {
    type: LinkedListTest,
    name: "linked_list_test",
    author: "Luca Saverio Esposito",
    description: "Testing the linked list operations",
    license: "GPL v2",
}

struct LinkedListTest;

const LIST_SIZE: u32 = 1000000; 

impl kernel::Module for LinkedListTest {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Starting Linked List Operations Test...\n");

        // Initialize list head
        let mut head = ListHead::new_uninitialized();
        head.init();

        // Measure adding elements
        let start_add = Ktime::ktime_get();
        for i in 0..LIST_SIZE {
            let mut item = Box::new(MyListItem::new(i), GFP_KERNEL).unwrap();
            head.add(item.get_list_head()); // Pass *mut ListHead
        }
        let end_add = Ktime::ktime_get();
        let duration_add = time::ktime_ms_delta(end_add, start_add);
        pr_info!("Time taken to add {} elements: {} ms\n", LIST_SIZE, duration_add);
        //pr_info!("Time taken to add 1000 elements: {} ms (start: {}, end: {})\n",duration_add,start_add.to_ms(),end_add.to_ms());

        // Measure iterating over elements
        let start_iter = Ktime::ktime_get();
        let mut iter = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);
        
        while let Some(item_head) = iter.next() {
            unsafe {
                let item = MyListItem::from_list_head(item_head as *mut ListHead);
                (*item).data += 1; // Add 1 to the data field of MyListItem
            }
        }

        let end_iter = Ktime::ktime_get();
        let duration_iter = time::ktime_ms_delta(end_iter, start_iter);
        pr_info!("Time taken to iterate over {} elements: {} ms\n", LIST_SIZE, duration_iter);

        // Measure removing elements
        let start_del = Ktime::ktime_get();
        let mut current_iter = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);

        // Directly delete entries as we iterate
        while let Some(entry_ptr) = current_iter.next() {
            head.del(entry_ptr);  // Pass the raw pointer directly
        }

        let end_del = Ktime::ktime_get();
        let duration_del = time::ktime_ms_delta(end_del, start_del);
        pr_info!("Time taken to remove {} elements: {} ms\n", LIST_SIZE,duration_del);

        // Test additional linked list operations
        let mut item = Box::new(MyListItem::new(1001), GFP_KERNEL).unwrap();
        head.add(item.get_list_head());
        let mut item = Box::new(MyListItem::new(1003), GFP_KERNEL).unwrap();
        head.add(item.get_list_head());

        // Check if the list is empty after adding one item
        assert!(!head.is_empty());
        pr_info!("List is not empty after adding one item.\n");

        // Replace the item
        let mut new_item = Box::new(MyListItem::new(1002), GFP_KERNEL).unwrap();
        head.replace(item.get_list_head(), new_item.get_list_head());

        // Verify the replacement
        let mut iter_after_replace = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);  
        if let Some(entry_ptr) = iter_after_replace.next() {
            let my_item = MyListItem::from_list_head(head as *mut bindings::list_head);
            pr_info!("Other items with data: {}\n", (*my_item).data);
        }

        pr_info!("Linked List Operations Test Completed.\n");
        Ok(LinkedListTest)
    }
}

/// A simple structure containing a `ListHead` for testing.
struct MyListItem {
    list: ListHead,
    data: u32,
}

impl MyListItem {
    fn new(data: u32) -> Self {
        let mut item = MyListItem {
            list: ListHead::new_uninitialized(),
            data,
        };
        item.list.init();
        item
    }
}

impl ListEntry for MyListItem {
    unsafe fn from_list_head(ptr: *mut bindings::list_head) -> *const Self {
        container_of!(ptr, MyListItem, list) as *const MyListItem
    }

    fn get_list_head(&mut self) -> *mut ListHead {
        &mut self.list as *mut ListHead
    }
}
