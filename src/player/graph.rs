use std::sync::{Arc, Weak};
use super::misc::atomic_vec::AtomicInitVec;
use std::cell::UnsafeCell;

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

    pub fn children(&self) -> &[NodeRef<T>] {
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
            sum += 1 + child.get().tree_size();
        }
        sum
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
        if !self.get_mut().children.init(children) {
            // TODO if here children were not added. pass this info up!
        } else {
            for child in self.get_mut().children() {
                child.get_mut().parent = Some(WeakNodeRef(Arc::downgrade(&self.0.clone())));
            }
        }
    }

    pub fn get(&self) -> &Node<T> {
        unsafe {
            &*self.0.get()
        }
    }

    pub fn get_mut(&self) -> &mut Node<T> {
        unsafe {
            &mut *self.0.get()
        }
    }
}

impl<T> WeakNodeRef<T> {
    pub fn upgrade(&self) -> NodeRef<T> {
        NodeRef(self.0.upgrade().unwrap())
    }
}
