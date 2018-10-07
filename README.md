# radix
IP CIDR prefix Radix Tree implementation in Rust

Note: Currently only supports IPv4 addresses, will eventually support IPv6 as well. Pull Requests welcome! :)

## Using
```
let mut tree = PrefixTree::new();

tree.add_cidr("10.1.1.0/24");
tree.add_cidr("10.5.0.0/16");

tree.contains_addr_v4_str("10.1.1.54").unwrap() // => true
tree.contains_addr_v4_str("10.5.99.101").unwrap() // => true
tree.contains_addr_v4_str("10.6.1.2").unwrap() // => false
```

