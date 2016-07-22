use std::sync::{Arc, Weak};
use super::misc::atomic_vec::AtomicInitVec;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Node<T> {
    data: T,
    parent: Option<WeakNodeRef<T>>,
    children: AtomicInitVec<NodeRef<T>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            data: data,
            parent: None,
            children: AtomicInitVec::new(),
        }
    }

    pub fn parent(&self) -> Option<&WeakNodeRef<T>> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> &mut [NodeRef<T>] {
        self.children.slice()
    }

    pub fn orphan(&mut self) {
        self.parent = None;
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn tree_size(&self) -> usize {
        let mut sum = 0;
        for child in self.children() {
            sum += 1 + child.tree_size();
        }
        sum
    }
}

impl<T> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Debug)]
pub struct NodeRef<T>(Arc<UnsafeCell<Node<T>>>);
#[derive(Debug)]
pub struct WeakNodeRef<T>(Weak<UnsafeCell<Node<T>>>);

unsafe impl<T> Send for NodeRef<T> { }
unsafe impl<T> Send for WeakNodeRef<T> { }

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        NodeRef(self.0.clone())
    }
}
impl<T> Clone for WeakNodeRef<T> {
    fn clone(&self) -> Self {
        WeakNodeRef(self.0.clone())
    }
}

impl<T> NodeRef<T> {
    pub fn new(data: T) -> NodeRef<T> {
        NodeRef(Arc::new(UnsafeCell::new(Node::new(data))))
    }

    pub fn add_children(&self, children: Vec<NodeRef<T>>) {
        if !self.children.init(children) {
            // TODO if here children were not added. pass this info up!
        } else {
            for child in self.children() {
                child.parent = Some(WeakNodeRef(Arc::downgrade(&self.0.clone())));
            }
        }
    }
}

impl<T> WeakNodeRef<T> {
    pub fn upgrade(&self) -> NodeRef<T> {
        NodeRef(self.0.upgrade().unwrap())
    }
}

impl<T> Deref for NodeRef<T> {
    type Target = Node<T>;
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0.get()
        }
    }
}

impl<T> DerefMut for NodeRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0.get()
        }
    }
}
