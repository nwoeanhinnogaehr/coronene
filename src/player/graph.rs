use std::sync::{Arc, Weak, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct Node<T> {
    data: T,
    parent: Option<WeakNodeRef<T>>,
    children: Vec<NodeRef<T>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Node<T> {
        Node {
            data: data,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn parent(&self) -> Option<&WeakNodeRef<T>> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> &[NodeRef<T>] {
        &self.children
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

#[derive(Clone, Debug)]
pub struct NodeRef<T>(Arc<RwLock<Node<T>>>);
#[derive(Clone, Debug)]
pub struct WeakNodeRef<T>(Weak<RwLock<Node<T>>>);

impl<T> NodeRef<T> {
    pub fn new(data: T) -> NodeRef<T> {
        NodeRef(Arc::new(RwLock::new(Node::new(data))))
    }

    pub fn add_child(&self, child: NodeRef<T>) -> NodeRef<T> {
        self.get_mut().children.push(NodeRef(child.0.clone()));
        child.get_mut().parent = Some(WeakNodeRef(Arc::downgrade(&self.0.clone())));
        child
    }

    pub fn get(&self) -> RwLockReadGuard<Node<T>> {
        self.0.read().unwrap()
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<Node<T>> {
        self.0.write().unwrap()
    }
}

impl<T> WeakNodeRef<T> {
    pub fn upgrade(&self) -> NodeRef<T> {
        NodeRef(self.0.upgrade().unwrap())
    }
}
