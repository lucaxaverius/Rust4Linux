// linked_list_test.rs

// SPDX-License-Identifier: GPL-2.0

//! Linked List Test Module

use kernel::{prelude::*, time::{self, Ktime}, linked_list::*};
use kernel::container_of;
//use core::marker::PhantomData;

module! {
    type: LinkedListTest,
    name: "linked_list_test",
    author: "Luca Saverio Esposito",
    description: "Testing the linked list operations",
    license: "GPL v2",
}

struct LinkedListTest;

const LIST_SIZE: usize = 10; 

impl kernel::Module for LinkedListTest {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Starting Linked List Operations Test...\n");

        // Initialize list head
        let mut head = ListHead::new_uninitialized();
        head.init();

        // Create a Vec to hold MyListItem instances
        let mut items = Vec::new();

        // Measure adding elements
        let start_add = Ktime::ktime_get();
        for i in 0..LIST_SIZE {
            let mut item = Box::new(MyListItem::new(i as u32), GFP_KERNEL).unwrap();
            //pr_info!("Added item {:p} with data: {}\n", item,item.data);
            head.add(item.get_list_head()); // Pass *mut ListHead
            let _ = items.push(item,GFP_KERNEL);
        }

        let end_add = Ktime::ktime_get();
        let duration_add = time::ktime_ms_delta(end_add, start_add);
        pr_info!("Time taken to add {} elements: {} ms\n", LIST_SIZE, duration_add);
        //pr_info!("Time taken to add {} elements: {} ms (start: {}, end: {})\n",LIST_SIZE,duration_add,start_add.to_ms(),end_add.to_ms());
        
        // Measure iterating over elements
        let start_iter = Ktime::ktime_get();
        let mut iter = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);
        
        while let Some(item) = iter.next() {
            item.data += 1; // Add 1 to the data field of MyListItem
            pr_info!("Item {:p}, with data: {}\n", item, item.data);               

        }

        let end_iter = Ktime::ktime_get();
        let duration_iter = time::ktime_ms_delta(end_iter, start_iter);
        pr_info!("Time taken to iterate over {} elements: {} ms\n", LIST_SIZE, duration_iter);
        //pr_info!("Time taken to iterate over {} elements: {} ms (start: {}, end: {})\n",LIST_SIZE,duration_iter,start_iter.to_ns(),end_iter.to_ns());
            
        // Measure removing elements
        let start_del = Ktime::ktime_get();
        let mut current_iter = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);

        // Directly delete entries as we iterate
        while let Some(to_delete) = current_iter.next() {
            head.del(to_delete.get_list_head()); 
        }

        let end_del = Ktime::ktime_get();
        let duration_del = time::ktime_ms_delta(end_del, start_del);
        pr_info!("Time taken to remove {} elements: {} ms\n", LIST_SIZE,duration_del);

        // Check if the list is empty after removing all the items
        assert!(head.is_empty());
        pr_info!("List is empty after removing all the items.\n");


        // Test additional linked list operations
        let mut item = Box::new(MyListItem::new(1001), GFP_KERNEL).unwrap();
        head.add(item.get_list_head());
        let mut item = Box::new(MyListItem::new(1003), GFP_KERNEL).unwrap();
        head.add(item.get_list_head());

        // Check if the list is empty after adding two item
        assert!(!head.is_empty());
        pr_info!("List is not empty after adding one item.\n");

        // Replace the item
        let mut new_item = Box::new(MyListItem::new(1002), GFP_KERNEL).unwrap();
        head.replace(item.get_list_head(), new_item.get_list_head());

        // Verify the replacement
        let mut iter_after_replace = ListIterator::<MyListItem>::new(&mut head as *mut ListHead);  
        while let Some(item_replaced) = iter_after_replace.next() {   
            pr_info!("Item with data: {}\n", item_replaced.data);
        }

        pr_info!("Linked List Operations Test Completed.\n");
        Ok(LinkedListTest)
    }
}

impl Drop for LinkedListTest {
    fn drop(&mut self) {
        pr_info!("Module unloaded\n");
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
            data: data,
        };
        item.list.init();
        item
    }
    fn get_data(&mut self) -> u32 {
        self.data
    }
}

impl ListEntry for MyListItem{
    unsafe fn parent_from_list_head(ptr: *mut ListHead) -> *mut Self {
        container_of!(ptr, MyListItem, list) as *mut MyListItem
    }

    fn get_list_head(&mut self) -> *mut ListHead {
        &mut self.list as *mut ListHead
    }
}
