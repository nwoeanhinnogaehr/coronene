use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::mem;
use std::slice;

/// A vector which is initially empty, and can be initialized with data once atomically. Threads
/// should not assume that the data they pass to init has been inserted into the vector, since
/// another thread may have done so first.
#[derive(Debug)]
pub struct AtomicInitVec<T> {
    ptr: AtomicPtr<Vec<T>>,
}

impl<T> AtomicInitVec<T> {
    pub fn new() -> AtomicInitVec<T> {
        AtomicInitVec { ptr: AtomicPtr::new(ptr::null_mut()) }
    }

    /// Initialize the vector with the given items. If the vector already contains items, do
    /// nothing.
    ///
    /// Returns true if the vector was initialized with `items`, or false if the vector had
    /// previously been initialized.
    pub fn init(&self, items: Vec<T>) -> bool {
        let items_ptr = Box::into_raw(Box::new(items));

        loop {
            if self.ptr.load(Ordering::SeqCst) != ptr::null_mut() {
                unsafe {
                    mem::drop(Box::from_raw(items_ptr));
                }
                return false; // there's already items
            }
            if self.ptr.compare_and_swap(ptr::null_mut(), items_ptr, Ordering::SeqCst) ==
               ptr::null_mut() {
                return true;
            }
        }
    }

    /// Returns a slice into the vector. Since the vector cannot be reallocated, this reference
    /// will always live as long as the vector.
    ///
    /// If the vector has not been initialized, this will return a zero length slice.
    pub fn slice(&self) -> &mut [T] {
        let ptr = self.ptr.load(Ordering::SeqCst);
        if ptr == ptr::null_mut() {
            unsafe {
                // zero length slice probably doesn't need a valid ptr
                slice::from_raw_parts_mut(ptr::null_mut(), 0)
            }
        } else {
            unsafe { &mut (*ptr) }
        }
    }
}

impl<T> Drop for AtomicInitVec<T> {
    fn drop(&mut self) {
        loop {
            let ptr = self.ptr.load(Ordering::SeqCst);
            if ptr == ptr::null_mut() {
                return;
            }
            if self.ptr.compare_and_swap(ptr, ptr::null_mut(), Ordering::SeqCst) == ptr {
                unsafe {
                    mem::drop(Box::from_raw(ptr));
                }
                return;
            }
        }
    }
}

#[test]
fn test_atomic_vec() {
    let av = AtomicInitVec::<usize>::new();
    assert_eq!(av.slice().len(), 0);
    assert_eq!(av.slice(), &[]);
    assert!(av.init(vec![1, 2, 3, 4, 5, 6]));
    assert_eq!(av.slice().len(), 6);
    assert_eq!(av.slice(), &[1, 2, 3, 4, 5, 6]);
    assert!(!av.init(vec![4, 3, 2, 1])); // cannot re-init
    assert_eq!(av.slice().len(), 6);
    assert_eq!(av.slice(), &[1, 2, 3, 4, 5, 6]);
}
