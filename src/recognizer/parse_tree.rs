use std::{rc::{Weak, Rc}, cell::RefCell};

#[derive(Debug, Clone)]
pub enum Child {
    Node(Rc<RefCell<Node>>),
    Leaf(Leaf),
}

pub type Leaf = String;

#[derive(Debug, Clone)]
pub struct Node {
    pub identifier: usize,
    pub rule_nr: usize,
    pub children: Vec<Child>,
    pub parent: Option<Weak<RefCell<Node>>>,
}

pub trait ParseTree {
    fn new(identifier: usize, rule_nr: usize) -> Self;
    fn add_node(&self, identifier: usize, rule_nr: usize) -> Option<Tree>;
    fn add_leaf(&self, value: &str);
    fn parent(&self) -> Option<Tree>;
    fn child(&self, identifier: usize) -> Option<Child>;
}

pub type Tree = Rc<RefCell<Node>>;

impl ParseTree for Tree {
    fn new(identifier: usize, rule_nr: usize) -> Self {
        let node = Node {
            identifier,
            rule_nr,
            children: Vec::new(),
            parent: None,
        };
        Rc::new(RefCell::new(node))
    }
    fn add_node(
        &self,
        identifier: usize,
        rule_nr: usize,
    ) -> Option<Rc<RefCell<Node>>> {
        let mut node = self.as_ref().borrow_mut();
        let mut new_node = Node {
            identifier,
            rule_nr,
            children: Vec::new(),
            parent: None,
        };
        new_node.parent = Some(Rc::downgrade(self));
        let new_ref = Rc::new(RefCell::new(new_node));
        node.children.push(Child::Node(new_ref.clone()));
        Some(new_ref)
    }
    fn add_leaf(&self, value: &str) {
        let mut node = self.as_ref().borrow_mut();
        let new_node = value.to_string();
        node.children.push(Child::Leaf(new_node));
    }
    
    fn parent(&self) -> Option<Rc<RefCell<Node>>> {
        let node = self.as_ref().borrow();
        node.parent.as_ref().and_then(|p| p.upgrade())
    }
    
    fn child(&self, identifier: usize) -> Option<Child> {
        let mut node = self.as_ref().borrow_mut();
        node.children.get_mut(identifier).cloned()
    }
}