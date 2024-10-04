// linked_list.rs

use core::marker::PhantomData;
use kernel::bindings;
use crate::pr_info;


/// Represents a kernel linked list head.
///
/// This struct corresponds to the `struct list_head` in the Linux kernel.
/// It provides methods to manipulate the linked list.
/// Al the examples are based on a simple struct defined as follows:
/// ```rust
/// struct MyListItem {
///    list: ListHead,
///    data: u32,
///}
/// ```
#[repr(C)]
pub struct ListHead {
    /// Pointer to the next element in the list.
    pub next: *mut bindings::list_head,
    /// Pointer to the previous element in the list.
    pub prev: *mut bindings::list_head,
}

impl ListHead {
    /// Creates a new, uninitialized `ListHead`.
    ///
    /// # Safety
    ///
    /// This function returns a `ListHead` with null `next` and `prev` pointers.
    /// The caller must ensure that this is properly initialized using `init` before
    /// it is used, to avoid undefined behavior.

    pub const fn new_uninitialized() -> Self {
        ListHead {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }

    /// Initializes the list head.
    ///
    /// This corresponds to the `INIT_LIST_HEAD` macro in the Linux kernel.
    /// It will initialize the `next` and `prev` pointers to point to itself.
    ///
    /// # Safety
    //
    /// This function is safe to use as long as the `ListHead` is not null.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut head = ListHead::new_uninitialized();
    /// head.init();
    /// ```
    pub fn init(&mut self) {
        unsafe {
            bindings::init_list_head(self as *mut ListHead as *mut bindings::list_head);
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
    /// # Safety
    ///
    /// The `new` parameter must not be null and must point to a valid, initialized `ListHead`.
    /// The list head on which `add` is called must also be properly initialized.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut item = Box::new(MyListItem::new(1001),GFP_KERNEL);
    /// head.add(&mut item.list_head);
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
    /// # Safety
    ///
    /// The `new` parameter must not be null and must point to a valid, initialized `ListHead`.
    /// The list head on which `add_tail` is called must also be properly initialized.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut item = Box::new(MyListItem::new(1001),GFP_KERNEL);
    /// head.add_tail(&mut item.list_head);
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
    /// # Safety
    ///
    /// The `entry` parameter must point to a valid `ListHead` that is part of a list.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut to_delete = Box::new(MyListItem::new(1001),GFP_KERNEL);
    /// head.add(&mut to_delete.list_head);
    /// head.del(&mut to_delete.list_head);
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
    /// # Safety
    ///
    /// Both `old` and `new` must point to valid, initialized `ListHead` elements.
    /// The `old` entry must be part of a list, and the `new` entry should not already
    /// be part of any list, or else the list structure may become corrupt.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut old_entry = Box::new(MyListItem::new(1001),GFP_KERNEL);
    /// head.add(&mut old_entry.list_head);
    /// let mut new_entry = Box::new(MyListItem::new(1002),GFP_KERNEL);
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
    /// # Safety
    ///
    /// Both `old` and `new` must point to valid `ListHead` elements. 
    /// The `old` entry must be part of a list, and the `new` entry should not be part 
    /// of any list to avoid list corruption. 
    /// After the operation, the `old` entry is reinitialized and may be reused.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut old_entry = Box::new(MyListItem::new(1001),GFP_KERNEL);
    /// head.add(&mut old_entry.list_head);
    /// let mut new_entry = Box::new(MyListItem::new(1002),GFP_KERNEL);
    /// head.replace(&mut old_entry, &mut new_entry);
    /// old_entry.value = 1003;
    /// head.add_tail(&mut old_entry.list_head);
    /// ```
    pub fn replace_init(&mut self, old: *mut ListHead, new: *mut ListHead) {
        unsafe {
            bindings::list_replace_init(
                old as *mut bindings::list_head,
                new as *mut bindings::list_head,
            );
        }
    }

    /// Moves an entry (also from another list) to the start of the list.
    ///
    /// This corresponds to the `list_move` function in the Linux kernel.
    ///
    /// # Arguments
    ///
    /// * `entry` - Pointer to the list entry to move.
    ///
    /// # Safety
    ///
    /// The `entry` parameter must point to a valid `ListHead` that is part of a list.
    ///
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
    /// # Safety
    ///
    /// The `entry` parameter must point to a valid `ListHead` that is part of a list.
    ///
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
    /// # Safety
    ///
    /// This function is safe to call as long as the list head has been initialized.
    /// It will return a boolean indicating whether the list is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// if head.is_empty() {
    ///     // Handle empty list
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        unsafe { 
            bindings::list_empty(self as *const ListHead as *mut bindings::list_head) != 0 
        }
    }

    /// Splices two lists.
    ///
    /// This corresponds to the `list_splice` function in the Linux kernel.
    /// it joins two lists, this is designed for stacks
    ///
    /// # Arguments
    ///
    /// * `list` - Pointer to the source list.
    /// * `prev` - Pointer to the place to add it in the destination list.
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

    /// Splices two lists.
    ///
    /// This corresponds to the `list_splice_tail` function in the Linux kernel.
    /// it joins two lists, each list being a queue.
    ///
    /// # Arguments
    ///
    /// * `list` - Pointer to the source list.
    /// * `prev` - Pointer to the place to add it in the destination list.
    ///
    /// # Example
    ///
    /// ```rust
    /// head.splice_init(&mut source_list, &mut prev_position);
    /// ```
    pub fn splice_tail(&mut self, list: *mut ListHead, prev: *mut ListHead) {
        unsafe {
            bindings::list_splice_tail(
                list as *mut bindings::list_head,
                prev as *mut bindings::list_head,
            );
        }
    }
}
/// Trait to associate a struct with its `ListHead` member.
///
/// This trait provides methods to retrieve the parent struct and its `ListHead` pointer.
pub trait ListEntry {
    /// Converts a `ListHead` pointer to a pointer of the parent struct.
    ///
    /// # Safety
    ///
    /// - The `ptr` must be a valid pointer to a `ListHead` that is embedded within a `Self` instance.
    /// - The memory referenced by `ptr` must be valid for the lifetime of `Self`.
    unsafe fn parent_from_list_head(ptr: *mut ListHead) -> *mut Self;

    /// Given a mut reference to `Self`, returns a pointer to its `ListHead` field.
    ///
    /// # Returns
    ///
    /// A mutable pointer to the `ListHead` field within `Self`.
    fn get_list_head(&mut self) -> *mut ListHead;
}

/// Iterator for traversing a linked list in forward order.
/// Yields raw pointers to `ListHead`.
pub struct ListIterator<'a, T: ListEntry> {
    current: *mut ListHead,
    head: *mut ListHead,
    _marker: PhantomData<&'a mut T>, // Tied to the lifetime of T
}

impl<'a, T: ListEntry> ListIterator<'a, T> {
    /// Creates a new `ListIterator`.
    ///
    /// # Arguments
    ///
    /// * `head` - Mutable pointer to the list head.
    pub fn new(head: *mut ListHead) -> Self {
        unsafe{
            ListIterator {
                current: (*head).next as *mut ListHead,
                head: head,
                _marker: PhantomData,
            }
        }
    }
}

impl<'a, T: ListEntry> Iterator for ListIterator<'a, T> {
    type Item = &'a mut T; // Return mutable reference

    /// A safe method to use when modifying the list, e.g., removing elements.
    /// or when simply iterating on it.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.head {
            //pr_info!("Iteration completed");
            return None;
        }
        
        if self.current.is_null() {
            pr_info!("Can't reference NULL pointer");
            return None;
        }
        //pr_info!("Current: {:?}, Head: {:?}\n", self.current, self.head);
        unsafe {
            let next = (*self.current).next as *mut ListHead;  // Get next before modifying current

            let ptr = T::parent_from_list_head(self.current);

            //pr_info!("MyListItem pointer is: {:p}",ptr);
            
            self.current = next;

            if ptr.is_null() {
                None
            } else {
                // Cast the pointer to the actual type T 
                Some(&mut *(ptr as *mut T))            
            }
        }
    }
}

/// Reverse iterator for traversing a linked list in reverse order.
/// Yields raw pointers to `ListHead`.
pub struct ReverseListIterator<'a, T: ListEntry> {
    current: *mut ListHead,
    head: *mut ListHead,
    _marker: PhantomData<&'a mut T>, // Tied to the lifetime of T
}

impl<'a, T: ListEntry> ReverseListIterator<'a, T> {
    /// Creates a new `ReverseListIterator`.
    ///
    /// # Arguments
    ///
    /// * `head` - Mutable pointer to the list head.
      
    pub fn new(head: *mut ListHead) -> Self {
        unsafe{ 
            ReverseListIterator {
                current: (*head).prev as *mut ListHead,
                head: head,
                _marker: PhantomData,
            }
        }
    }  
    
}

impl<'a, T: ListEntry> Iterator for ReverseListIterator<'a, T> {
    type Item = &'a mut T; // Yield raw pointer

    /// A safe method to use when modifying the list, e.g., removing elements.
    /// or when simply iterating on it.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.head{
            //pr_info!("Iteration completed");
            return None;
        }
        if self.current.is_null() {
            //pr_info!("Can't reference NULL pointer");
            return None;
        }
        //pr_info!("Current: {:?}, Head: {:?}\n", self.current, self.head);
        unsafe {
            let prev = (*self.current).prev as *mut ListHead;  // Get next before modifying current

            
            let ptr = T::parent_from_list_head(self.current);
            self.current = prev; //move to previous node

            if ptr.is_null() {
                None
            } else {
                Some(&mut *ptr)
            }
        }
    }

}
