use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::Deref;

pub struct Node<T: Clone> {
    pub data: T,
    incoming: Vec<NodeRef<T>>,
    outgoing: Vec<NodeRef<T>>,
}

impl<T: Clone> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node {
            data: data,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    pub fn incoming(&self) -> &[NodeRef<T>] {
        &self.incoming
    }

    pub fn outgoing(&self) -> &[NodeRef<T>] {
        &self.outgoing
    }
}

#[derive(Clone)]
pub struct NodeRef<T: Clone>(Arc<RwLock<Node<T>>>);

impl<T: Clone> Deref for NodeRef<T> {
    type Target = Arc<RwLock<Node<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> NodeRef<T> {
    pub fn new(data: T) -> NodeRef<T> {
        NodeRef(Arc::new(RwLock::new(Node::new(data))))
    }

    pub fn add_child(&self, child: NodeRef<T>) -> NodeRef<T> {
        self.write().unwrap().outgoing.push(child.clone());
        child.write().unwrap().incoming.push(self.clone());
        child
    }

    pub fn node(&self) -> RwLockReadGuard<Node<T>> {
        self.read().unwrap()
    }

    pub fn node_mut(&self) -> RwLockWriteGuard<Node<T>> {
        self.write().unwrap()
    }
}

pub struct Graph<T: Clone> {
    root: NodeRef<T>,
}

impl<T: Clone> Graph<T> {
    pub fn new(root: NodeRef<T>) -> Graph<T> {
        Graph { root: root }
    }

    pub fn root(&self) -> NodeRef<T> {
        self.root.clone()
    }
}
