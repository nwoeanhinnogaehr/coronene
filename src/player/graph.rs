use std::sync::{Arc, Weak};
use super::misc::atomic_vec::AtomicInitVec;
use std::cell::UnsafeCell;
use dot;

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
            if child.get().children().len() > 0 {
                sum += 1 + child.get().tree_size();
            }
        }
        sum
    }
}

#[derive(Debug)]
pub struct NodeRef<T>(Arc<UnsafeCell<Node<T>>>);
#[derive(Debug)]
pub struct WeakNodeRef<T>(Weak<UnsafeCell<Node<T>>>);

unsafe impl<T> Send for NodeRef<T> {}
unsafe impl<T> Send for WeakNodeRef<T> {}

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
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut Node<T> {
        unsafe { &mut *self.0.get() }
    }

    pub fn render(&self, file: String) {
        use std::fs::File;
        use std::collections::VecDeque;
        use rand::{Rng, thread_rng};
        let mut edges = Vec::new();
        let mut q = VecDeque::new();
        q.push_back((self, 0));
        while let Some((node, n)) = q.pop_front() {
            for (i, child) in node.get().children().iter().enumerate() {
                if child.get().children().len() > 0 {
                    let id = thread_rng().gen();
                    q.push_back((child, id));
                    edges.push((n, id));
                }
            }
        }
        dot::render(&Edges(edges), &mut File::create(file).unwrap());
    }
}

impl<T> WeakNodeRef<T> {
    pub fn upgrade(&self) -> NodeRef<T> {
        NodeRef(self.0.upgrade().unwrap())
    }
}

type Nd = usize;
type Ed = (usize, usize);
struct Edges(Vec<Ed>);

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example1").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", *n)).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges(ref v) = self;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s, t) in v.iter() {
            nodes.push(s);
            nodes.push(t);
        }
        nodes.sort();
        nodes.dedup();
        nodes.into()
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        let &Edges(ref edges) = self;
        (&edges[..]).into()
    }

    fn source(&self, e: &Ed) -> Nd {
        let &(s, _) = e;
        s
    }

    fn target(&self, e: &Ed) -> Nd {
        let &(_, t) = e;
        t
    }
}
