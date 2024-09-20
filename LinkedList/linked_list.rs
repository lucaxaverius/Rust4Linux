// SPDX-License-Identifier: GPL-2.0

//! Rust abstractions for kernel linked list functions.
//!
//! This module provides safe Rust bindings for the Linux kernel's linked list
//! implementation, allowing Rust code to interact with kernel linked lists.
//! It includes various list operations and iterators to traverse the lists.

use core::marker::PhantomData;
use kernel::prelude::*;
use crate::container_of;
use kernel::bindings;

/// Represents a kernel linked list head.
///
/// This struct corresponds to the `struct list_head` in the Linux kernel.
/// It provides methods to manipulate the linked list.
#[repr(C)]
pub struct ListHead {
    pub next: *mut bindings::list_head,
    pub prev: *mut bindings::list_head,
}

impl ListHead {
    /// Creates a new, uninitialized `ListHead`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `ListHead` is properly initialized
    /// using `INIT_LIST_HEAD` before use.
    pub const fn new_uninitialized() -> Self {
        ListHead {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }

    /// Initializes the list head.
    ///
    /// This corresponds to the `INIT_LIST_HEAD` macro in the Linux kernel.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut head = ListHead::new_uninitialized();
    /// head.init();
    /// ```
    pub fn init(&mut self) {
        unsafe {
            bindings::INIT_LIST_HEAD(self as *mut ListHead as *mut bindings::list_head);
        }
    }

    /// Adds a new entry after the specified head.
    ///
    /// This corresponds to the `list_add` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `new` - Pointer to the new list entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.add(&mut new_entry);
    /// ```
    pub fn add(&mut self, new: *mut ListHead) {
        unsafe {
            bindings::list_add(
                new as *mut bindings::list_head,
                self as *mut ListHead as *mut bindings::list_head,
            );
        }
    }

    /// Adds a new entry before the specified head.
    ///
    /// This corresponds to the `list_add_tail` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `new` - Pointer to the new list entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.add_tail(&mut new_entry);
    /// ```
    pub fn add_tail(&mut self, new: *mut ListHead) {
        unsafe {
            bindings::list_add_tail(
                new as *mut bindings::list_head,
                self as *mut ListHead as *mut bindings::list_head,
            );
        }
    }

    /// Deletes an entry from the list.
    ///
    /// This corresponds to the `list_del` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `entry` - Pointer to the list entry to delete.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.del(&mut entry);
    /// ```
    pub fn del(&mut self, entry: *mut ListHead) {
        unsafe {
            bindings::list_del(
                entry as *mut bindings::list_head,
            );
        }
    }

    /// Replaces an old entry with a new entry in the list.
    ///
    /// This corresponds to the `list_replace` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `old` - Pointer to the old list entry to be replaced.
    /// * `new` - Pointer to the new list entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.replace(&mut old_entry, &mut new_entry);
    /// ```
    pub fn replace(&mut self, old: *mut ListHead, new: *mut ListHead) {
        unsafe {
            bindings::list_replace(
                old as *mut bindings::list_head,
                new as *mut bindings::list_head,
            );
        }
    }

    /// Replaces an old entry with a new entry and reinitializes the old entry.
    ///
    /// This corresponds to the `list_replace_init` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `old` - Pointer to the old list entry to be replaced.
    /// * `new` - Pointer to the new list entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.replace_init(&mut old_entry, &mut new_entry);
    /// ```
    pub fn replace_init(&mut self, old: *mut ListHead, new: *mut ListHead) {
        unsafe {
            bindings::list_replace_init(
                old as *mut bindings::list_head,
                new as *mut bindings::list_head,
            );
        }
    }

    /// Moves an entry to the start of the list.
    ///
    /// This corresponds to the `list_move` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `entry` - Pointer to the list entry to move.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.move_to_start(&mut entry);
    /// ```
    pub fn move_to_start(&mut self, entry: *mut ListHead) {
        unsafe {
            bindings::list_move(
                entry as *mut bindings::list_head,
                self as *mut ListHead as *mut bindings::list_head,
            );
        }
    }

    /// Moves an entry to the end of the list.
    ///
    /// This corresponds to the `list_move_tail` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `entry` - Pointer to the list entry to move.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.move_to_end(&mut entry);
    /// ```
    pub fn move_to_end(&mut self, entry: *mut ListHead) {
        unsafe {
            bindings::list_move_tail(
                entry as *mut bindings::list_head,
                self as *mut ListHead as *mut bindings::list_head,
            );
        }
    }

    /// Checks if the list is empty.
    ///
    /// This corresponds to the `list_empty` function in the Linux kernel.
    ///
    /// # Returns
    ///
    /// `true` if the list is empty, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// if head.is_empty() {
    ///     // Handle empty list
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        unsafe { bindings::list_empty(self as *const ListHead as *const bindings::list_head) != 0 }
    }

    /// Splices two lists.
    ///
    /// This corresponds to the `list_splice` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `list` - Pointer to the source list.
    /// * `prev` - Pointer to the position in the destination list to splice into.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.splice(&mut source_list, &mut prev_position);
    /// ```
    pub fn splice(&mut self, list: *mut ListHead, prev: *mut ListHead) {
        unsafe {
            bindings::list_splice(
                list as *mut bindings::list_head,
                prev as *mut bindings::list_head,
            );
        }
    }

    /// Splices two lists and reinitializes the source list.
    ///
    /// This corresponds to the `list_splice_init` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `list` - Pointer to the source list.
    /// * `prev` - Pointer to the position in the destination list to splice into.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.splice_init(&mut source_list, &mut prev_position);
    /// ```
    pub fn splice_init(&mut self, list: *mut ListHead, prev: *mut ListHead) {
        unsafe {
            bindings::list_splice_init(
                list as *mut bindings::list_head,
                prev as *mut bindings::list_head,
            );
        }
    }
}

/// Iterator for traversing a linked list in forward order.
///
/// This struct allows iterating over the entries of a `ListHead`.
pub struct ListIterator<'a, T> {
    current: *mut bindings::list_head,
    head: *const ListHead,
    member: fn(&T) -> &bindings::list_head,  // Function to get the list_head field
    _marker: PhantomData<&'a T>,
}


impl<'a, T> ListIterator<'a, T> {
    /// Creates a new `ListIterator`.
    ///
    /// # Arguments
    ///
    /// * `head` - Reference to the list head.
    ///
    /// # Example
    ///
    /// ```rust
    /// for entry in list.iter::<MyStruct>() {
    ///     // Use entry
    /// }
    /// ```
    pub fn new(head: &'a ListHead, member: fn(&T) -> &bindings::list_head) -> Self {
        ListIterator {
            current: head as *const ListHead as *mut bindings::list_head,
            head,
            member,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we are at the end of the list (back to the head)
        if self.current == self.head as *const ListHead as *mut bindings::list_head {
            return None;
        }

        unsafe {
            // Use container_of! with the member field passed during construction
            let ptr = container_of!(self.current, T, list) as *const T;

            // Move to the next list_head in the list
            self.current = (*self.current).next;

            // Return the parent structure reference if it's not null
            if ptr.is_null() {
                None
            } else {
                Some(&*ptr)
            }
        }
    }
}

/// Reverse iterator for traversing a linked list in reverse order.
///
/// This struct allows iterating over the entries of a `ListHead` in reverse.
///
/// PhantomData is a zero-sized type used to mark unused generic type parameters. 
/// It informs the Rust compiler about certain properties of your types without actually storing any data. 
/// This is crucial for maintaining correct type relationships, especially around ownership and lifetimes.
/// In details: helps manage lifetimes and borrowing rules. It ensures that the iterator doesn't outlive the data it references.
pub struct ReverseListIterator<'a, T> {
    current: *mut bindings::list_head,
    head: &'a ListHead,
    member: fn(&T) -> &bindings::list_head,  // Function to get the list_head field
    _marker: PhantomData<&'a T>,
}

impl<'a, T> ReverseListIterator<'a, T> {
    /// Creates a new `ReverseListIterator`.
    ///
    /// # Arguments
    ///
    /// * `head` - Reference to the list head.
    ///
    /// # Example
    ///
    /// ```rust
    /// for entry in list.iter_reverse::<MyStruct>() {
    ///     // Use entry
    /// }
    /// ```
    pub fn new(head: &'a ListHead, member: fn(&T) -> &bindings::list_head) -> Self {
        ReverseListIterator {
            current: head as *const ListHead as *mut bindings::list_head,
            head,
            member,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for ReverseListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.head as *const ListHead as *mut bindings::list_head {
            return None;
        }

        unsafe {
            // Use container_of! with the member field passed during construction
            let ptr = container_of!(self.current, T, list) as *const T;

            self.current = (*self.current).prev;

            if ptr.is_null() {
                None
            } else {
                Some(&*ptr)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_operations() {
        // Initialize list head
        let mut head = ListHead::new_uninitialized();
        head.init();
    
        // Create new entries
        let mut entry1 = Box::new(MyListItem::new(1));
        let mut entry2 = Box::new(MyListItem::new(2));
        let mut entry3 = Box::new(MyListItem::new(3));
    
        // Add entries to the list
        head.add(&mut entry1.list);
        head.add_tail(&mut entry2.list);
        head.add_tail(&mut entry3.list);
    
        // Check list is not empty
        assert!(!head.is_empty());
    
        // Replace entry2 with a new entry
        let mut entry4 = Box::new(MyListItem::new(4));
        head.replace(&mut entry2.list, &mut entry4.list);
    
        // Iterate over the list and verify entries
        let mut iter = ListIterator::<MyListItem>::new(&head, |item: &MyListItem| &item.list);
        assert_eq!(iter.next().unwrap().data, 1);
        assert_eq!(iter.next().unwrap().data, 4);
        assert_eq!(iter.next().unwrap().data, 3);
        assert!(iter.next().is_none());
    
        // Iterate in reverse and verify entries
        let mut reverse_iter = ReverseListIterator::<MyListItem>::new(&head, |item: &MyListItem| &item.list);
        assert_eq!(reverse_iter.next().unwrap().data, 3);
        assert_eq!(reverse_iter.next().unwrap().data, 4);
        assert_eq!(reverse_iter.next().unwrap().data, 1);
        assert!(reverse_iter.next().is_none());
    
        // Delete entry4
        head.del(&mut entry4.list);
        assert!(!head.is_empty());
    
        // Final iteration
        let mut final_iter = ListIterator::<MyListItem>::new(&head, |item: &MyListItem| &item.list);
        assert_eq!(final_iter.next().unwrap().data, 1);
        assert_eq!(final_iter.next().unwrap().data, 3);
        assert!(final_iter.next().is_none());
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
}

