// linked_list.rs

use core::marker::PhantomData;
use kernel::bindings;
use crate::pr_info;


/// Represents a kernel linked list head.
///
/// This struct corresponds to the `struct list_head` in the Linux kernel.
/// It provides methods to manipulate the linked list.
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
    /// The caller must ensure that the `ListHead` is properly initialized
    /// using `init` before use.
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
        unsafe { 
            bindings::list_empty(self as *const ListHead as *mut bindings::list_head) != 0 
        }
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
            pr_info!("Iteration completed");
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
