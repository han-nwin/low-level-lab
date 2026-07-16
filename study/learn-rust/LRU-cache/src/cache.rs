use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    next: Option<Rc<RefCell<Node>>>,   //pointer or None
    prev: Option<Weak<RefCell<Node>>>, // weak pointer or None
    value: Option<i64>,
    key: Option<u32>,
}

pub struct LRUCache {
    capacity: usize,
    map: HashMap<u32, Rc<RefCell<Node>>>, // key:val = number:Node_ptr
    head: Rc<RefCell<Node>>,              // pointer to data on heap
    tail: Rc<RefCell<Node>>,              // pointer to data on heap
}

impl LRUCache {
    pub fn new(capacity: usize) -> LRUCache {
        assert!(capacity > 0, "capacity has to be greater than 0");
        let map = HashMap::new();
        let head = Rc::new(RefCell::new(Node {
            next: None,
            prev: None,
            value: None,
            key: None,
        }));
        let tail = Rc::new(RefCell::new(Node {
            next: None,
            prev: None,
            value: None,
            key: None,
        }));

        head.borrow_mut().next = Some(Rc::clone(&tail));

        tail.borrow_mut().prev = Some(Rc::downgrade(&head)); // downgrade Rc -> Weak

        LRUCache {
            capacity,
            map,
            head: head.clone(), // dummy node
            tail: tail.clone(), // dummy node
        }
    }

    // get a node value
    pub fn get(&self, key: u32) -> Option<i64> {
        let node = self.map.get(&key)?; // Option<> here
        // Move it to the front
        Self::move_to_front(self, key);

        node.borrow().value // return the copied value. value is Option<usize> so it's Copy

        // if Value is a String we can do
        // let k = node.borrow();
        // let val = Ref::map(k, |node| node.value.as_ref().unwrap());
    }

    // put new node
    pub fn put(&mut self, key: u32, value: i64) {
        // Check if key exist, then just update val and move to front
        if self.map.contains_key(&key) {
            let node = self.map.get(&key);

            if let Some(node_rc) = &node {
                // update value
                node_rc.borrow_mut().value = Some(value);
            }
            Self::move_to_front(self, key);
        } else {
            // Check if cache is full then delete the LRU before inserting new node
            if self.map.len() >= self.capacity {
                // === Delete LRU node ===

                // Since capacity > 0 so tail won't be pointing to head here
                // get the node before the tail
                let lru_node = self
                    .tail
                    .borrow()
                    .prev
                    .as_ref()
                    .and_then(|weak| weak.upgrade());

                // if this node exist
                if let Some(lru_node_rc) = lru_node {
                    // point the tail to the node before the lru node
                    self.tail.borrow_mut().prev = lru_node_rc.borrow().prev.clone();

                    // point the node before the lru node to the tail
                    let node_before_lru = lru_node_rc
                        .borrow()
                        .prev
                        .as_ref()
                        .and_then(|weak| weak.upgrade());
                    if let Some(node_before_lru_rc) = node_before_lru {
                        node_before_lru_rc.borrow_mut().next = Some(Rc::clone(&self.tail));
                    }

                    // Delete it here
                    lru_node_rc.borrow_mut().next = None;
                    lru_node_rc.borrow_mut().prev = None;
                    // Delete from the map
                    if let Some(lru_node_key) = lru_node_rc.borrow().key {
                        self.map.remove(&lru_node_key);
                    }
                    drop(lru_node_rc); // drop the data
                }
            }
            // create new Node
            let new_node = Rc::new(RefCell::new(Node {
                next: None,
                prev: None,
                value: Some(value),
                key: Some(key),
            }));
            // get the current node in the front
            let old_node = match &self.head.borrow().next {
                Some(node_rc) => Some(Rc::clone(node_rc)),
                None => None,
            };

            self.head.borrow_mut().next = Some(Rc::clone(&new_node));
            new_node.borrow_mut().prev = Some(Rc::downgrade(&self.head));

            // if head->next exist
            if let Some(old_node_rc) = old_node {
                old_node_rc.borrow_mut().prev = Some(Rc::downgrade(&new_node));
                new_node.borrow_mut().next = Some(Rc::clone(&old_node_rc));
            }
            //==== End insert to linked list ===

            // Insert to map
            self.map.insert(key, new_node);
        }
    }

    // == Move to front helper
    fn move_to_front(&self, key: u32) {
        let node = self.map.get(&key);
        let front_node = self.head.borrow().next.clone();

        if let Some(node_rc) = &node {
            // if it's already at the front, do nothing
            if let Some(front_node_rc) = &front_node
                && Rc::ptr_eq(node_rc, front_node_rc)
            {
                return;
            }

            // Move it to the front
            // break the link
            let before = node_rc
                .borrow()
                .prev
                .as_ref()
                .and_then(|weak| weak.upgrade());
            let after = node_rc.borrow().next.clone();

            if let Some(before_rc) = &before {
                before_rc.borrow_mut().next = after.clone();
            }
            if let Some(after_rc) = &after {
                after_rc.borrow_mut().prev = before.as_ref().map(Rc::downgrade);
            }
            //connect to head
            let node_after_head = self.head.borrow().next.clone();
            node_rc.borrow_mut().next = node_after_head.clone();
            if let Some(node_after_head_rc) = node_after_head {
                node_after_head_rc.borrow_mut().prev = Some(Rc::downgrade(node_rc))
            }

            self.head.borrow_mut().next = Some(Rc::clone(node_rc));
            node_rc.borrow_mut().prev = Some(Rc::downgrade(&self.head));
        }
    }

    // == print content
    pub fn display(&self) {
        println!("Hash Map:");
        for (key, node_rc) in &self.map {
            println!("key: {:?}, value: {:?}", key, node_rc.borrow().value);
        }
        //print the linked list
        print!("\nLinked list:\n");
        let mut node = self.head.borrow().next.clone();
        while let Some(node_rc) = node {
            // if reach tail, break
            if Rc::ptr_eq(&node_rc, &self.tail) {
                break;
            }

            print!("{:?}", node_rc.borrow().value);

            let next = node_rc.borrow().next.clone();
            if let Some(next_rc) = &next
                && !Rc::ptr_eq(next_rc, &self.tail)
            {
                print!(" <-> ");
            }

            node = next;
        }
        println!();
    }
}
