
use std::net::{Ipv4Addr,IpAddr};
use std::fmt;

#[macro_use]
extern crate failure;
use failure::Error;

mod prefix;
use prefix::CidrPrefix;

pub struct Node<T> where T: Copy {
    left:   Option<Box<Node<T>>>,
    right:  Option<Box<Node<T>>>,
    val:    Option<T>
}

// TODO: implement for v6
impl <T: Copy> Node<T>
{
    fn new() -> Node<T>
    {
        Node::<T> { left: None, right: None, val: None }
    }

    fn insert(&mut self, key: u32, mask: u32, val: T)
    {
        let bit: u32 = 0x80000000;
        if mask == 0 {
            self.val = Some(val);
            return;
        }
        let next_node = if (key & bit) == 0 { &mut self.left } else { &mut self.right };
        match next_node {
            &mut Some(ref mut boxed_node) => boxed_node.insert(key << 1, mask << 1, val),
            &mut None => {
                // Out of this prefix, start extending
                // Will finally insert at base case (mask == 0)
                let mut new_node = Node::<T> { val: None, left: None, right: None};
                new_node.insert(key << 1, mask << 1, val);
                *next_node = Some(Box::new(new_node));
            }
        }
    }

    fn _find(&self, key: u32, mask: u32, cur_val: Option<T>) -> Option<T>
    {
        let bit: u32 = 0x80000000;
        if mask == 0 {
            return self.val.or(cur_val);
        }

        let next_node = if (key & bit) == 0 { &self.left } else { &self.right };
        match next_node {
            &Some(ref boxed_node) => boxed_node._find(key << 1, mask << 1, self.val.or(cur_val)),
            &None                 => self.val.or(cur_val),
        }
    }

    fn find(&self, key: u32, mask: u32) -> Option<T>
    {
        self._find(key, mask, None)
    }

}

impl fmt::Debug for Node<u8> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node [ val: {:?}, left: {:?}, right: {:?} ]", self.val, self.left, self.right)
    }
}

pub struct PrefixTree {
    root:   Node<u8>,
}

impl PrefixTree {
    pub fn new() -> PrefixTree
    {
        PrefixTree { root: Node::<u8>::new() }
    }

    pub fn add_prefix(&mut self, net: Ipv4Addr, mask: Ipv4Addr)
    {
        self.root.insert(u32::from(net), u32::from(mask), 1)
    }

    pub fn add_cidr(&mut self, cidr: &str) -> Result<(), Error>
    {
        let prefix = cidr.parse::<CidrPrefix>()?;

        match prefix.net {
            IpAddr::V4(net) => {
                let mask = (0xffffffff as u32) & !(((1 as u32) << (32 - prefix.prefix)) - 1);
                self.root.insert(u32::from(net), mask, 1);
                Ok(())
            },
            IpAddr::V6(_) => Err(format_err!("Unimplemented Ipv6")),
        }
    }

    pub fn contains_addr_v4(&self, addr: Ipv4Addr) -> bool
    {
        self.root.find(u32::from(addr), 0xffffffff).is_some()
    }

    pub fn contains_addr_v4_str(&self, addr: &str) -> Result<bool, Error>
    {
        let ip_addr: Ipv4Addr = addr.parse()?;
        Ok(self.contains_addr_v4(ip_addr))
    }
}


#[cfg(test)]
mod tests {
    use Node;
    use PrefixTree;
    #[test]
    fn test_node_insert_find() {
        let mut root = Node::<u32>::new();
        // Add 10.1.1.0/24 as 1
        let key = 0x0a010100 as u32;
        let mask = 0xffffff00 as u32;
        let val = 1;
        root.insert(key, mask, val);

        // Add 10.8.0.0/16 as 2
        root.insert(0x0a080000 as u32, 0xffff0000, 2);

        // Add 127.0.0.0/8 as 3
        root.insert(0x7f000000 as u32, 0xff000000, 3);

        // Verify that 10.1.1.0/24 is 1
        assert_eq!(root.find(key, mask), Some(1));

        // Check 10.1.1.1/32 is 1
        assert_eq!(root.find(key+1, 0xffffffff as u32), Some(1));

        // Check 10.1.1.255/32 is 1
        assert_eq!(root.find(0x0a0101ff, 0xffffffff), Some(1));

        // Check that 10.1.2.0/32 is not present
        assert_eq!(root.find(0x0a010200, 0xffffffff), None);

        // Check that 10.2.2.1 is not present
        assert_eq!(root.find(0x0a020201, 0xffffffff as u32), None);

        // Check that 10.8.127.3/32 is 2
        assert_eq!(root.find(0x0a087f03, 0xffffffff), Some(2));

        // Check that 10.9.127.3/32 is not present
        assert_eq!(root.find(0x0a097f03, 0xffffffff), None);

        // Check 127.255.0.255 is 3
        assert_eq!(root.find(0x7fff00ff, 0xffffffff), Some(3));

        // Check that 128.0.0.0/8 is None
        assert_eq!(root.find(0x80000000, 0xff000000), None);
    }

    #[test]
    fn test_basic() {
        let mut root = Node::<u8>::new();
        root.insert(0x0a010200, 0xffffff00, 1);

        assert_eq!(root.find(0x0a010201, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a010280, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a0102c0, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a0102e0, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a0102f0, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a0102ff, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a01027f, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a010255, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a0102dd, 0xffffffff), Some(1));
    }

    #[test]
    fn test_node_non_overlap() {
        let mut root = Node::<u8>::new();
        root.insert(0x0a010100, 0xffffff00, 1);
        root.insert(0x0a020000, 0xffff0000, 1);

        assert_eq!(root.find(0x0a010101, 0xffffffff), Some(1));
        assert_eq!(root.find(0x0a02ffff, 0xffffffff), Some(1));
    }

    #[test]
    fn test_node_overlap_basic() {
        let mut root = Node::<u8>::new();
        root.insert(0x50000000, 0xf0000000, 1);
        assert_eq!(root.find(0x500000ff, 0xffffffff), Some(1));


        root.insert(0x40000000, 0xc0000000, 2);

        assert_eq!(root.find(0x500000ff, 0xffffffff), Some(1));
        assert_eq!(root.find(0x400000ff, 0xffffffff), Some(2));
    }

    #[test]
    fn test_tree_find() {
        let mut tree = PrefixTree::new();
        assert!(tree.add_cidr("10.1.1.0/24").is_ok());

        assert!(tree.add_cidr("10.2.0.0/16").is_ok());

        assert_eq!(tree.contains_addr_v4_str("10.1.1.1").unwrap(), true);
        assert_eq!(tree.contains_addr_v4_str("10.2.255.255").unwrap(), true);
        assert_eq!(tree.contains_addr_v4_str("10.2.0.0").unwrap(), true);
        assert_eq!(tree.contains_addr_v4_str("10.3.0.0").unwrap(), false);
    }

    #[test]
    fn test_tree_overlap() {
        let mut tree = PrefixTree::new();
        assert!(tree.add_cidr("10.1.0.0/16").is_ok());

        assert_eq!(tree.contains_addr_v4_str("10.1.45.88").unwrap(), true);
        assert_eq!(tree.contains_addr_v4_str("10.1.50.22").unwrap(), true);

        assert!(tree.add_cidr("10.1.45.0/24").is_ok());
        assert_eq!(tree.contains_addr_v4_str("10.1.45.88").unwrap(), true);
        assert_eq!(tree.contains_addr_v4_str("10.1.50.22").unwrap(), true);
    }
}
