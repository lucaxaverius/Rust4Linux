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

impl kernel::Module for LinkedListTest {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Starting Linked List Operations Test...\n");

        // Initialize list head
        let mut head = ListHead::new_uninitialized();
        head.init();

        // Measure adding elements
        let start_add = Ktime::ktime_get();
        for i in 0..1000 {
            let item = Box::new(MyListItem::new(i), GFP_KERNEL);
            head.add(& mut item.unwrap().list);
        }
        let end_add = Ktime::ktime_get();
        let duration_add = time::ktime_ms_delta(end_add, start_add);
        pr_info!("Time taken to add 1000 elements: {} ms\n", duration_add);

        // Measure iterating over elements
        let start_iter = Ktime::ktime_get();
        let mut iter = ListIterator::<MyListItem>::new(&head);
        while iter.next().is_some() {}
        let end_iter = Ktime::ktime_get();
        let duration_iter = time::ktime_ms_delta(end_iter, start_iter);
        pr_info!("Time taken to iterate over 1000 elements: {} ms\n", duration_iter);

        // Measure removing elements
        let start_del = Ktime::ktime_get();
        let mut to_delete = Vec::new();
        let mut current_iter = ListIterator::<MyListItem>::new(&head);
        
        // Collect entries to delete
        while let Some(entry) = current_iter.next() {
            to_delete.push(entry, GFP_KERNEL);
        }
        
        // Now, remove them from the head
        for entry in to_delete {
            head.del(&mut entry.list);
        }
        let end_del = Ktime::ktime_get();
        let duration_del = time::ktime_ms_delta(end_del, start_del);
        pr_info!("Time taken to remove 1000 elements: {} ms\n", duration_del);
        
        // Test additional linked list operations
        let item = Box::new(MyListItem::new(1001), GFP_KERNEL)?;
        head.add(&mut item.list);

        // Check if the list is empty after adding one item
        assert!(!head.is_empty());
        pr_info!("List is not empty after adding one item.\n");

        // Replace the item
        let new_item = Box::new(MyListItem::new(1002), GFP_KERNEL)?;
        head.replace(&mut item.list, &mut new_item.list);

        // Verify the replacement
        let mut iter_after_replace = ListIterator::<MyListItem>::new(&head);
        let first_entry = iter_after_replace.next().unwrap();
        assert_eq!(first_entry.data, 1002);
        pr_info!("Item successfully replaced with data: {}\n", first_entry.data);

        pr_info!("Linked List Operations Test Completed.\n");
        Ok(LinkedListTest)
    }
}

// Define a simple structure containing a list_head for testing
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
}

