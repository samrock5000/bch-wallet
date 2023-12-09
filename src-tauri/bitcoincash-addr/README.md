# Rust Bitcoin Cash Address Library

[![Build Status](https://travis-ci.org/hlb8122/rust-bitcoincash-addr.svg?branch=master)](https://travis-ci.org/hlb8122/rust-bitcoincash-addr)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Cargo](https://img.shields.io/crates/v/bitcoincash-addr.svg)](https://crates.io/crates/bitcoincash-addr)
[![Documentation](https://docs.rs/bitcoincash-addr/badge.svg)](
https://docs.rs/bitcoincash-addr)

A simple library providing an `Address` struct enabling encoding/decoding of Bitcoin Cash addresses.

## Examples

### Convert Base58 to CashAddr

```rust
use bitcoincash_addr::{Address, Network, Scheme};

fn main() {
    // Decode base58 address
    let legacy_addr = "1NM2HFXin4cEQRBLjkNZAS98qLX9JKzjKn";
    let mut addr = Address::decode(legacy_addr).unwrap();

    // Change the base58 address to a test network cashaddr
    addr.network = Network::Test;
    addr.scheme = Scheme::CashAddr;

    // Encode cashaddr
    let cashaddr_str = addr.encode().unwrap();

    // bchtest:qr4zgpuznfg923ntyauyeh5v7333v72xhum2dsdgfh
    println!("{}", cashaddr_str);
}

```

### Encode from raw address

```rust
use bitcoincash_addr::Address;

fn main() {
    // Raw hash160 bytes
    let raw_address = [
        227, 97, 202, 154, 127, 153, 16, 124, 23, 166, 34, 224, 71, 227, 116, 93, 62, 25, 207, 128,
        78, 214, 60, 92, 64, 198, 186, 118, 54, 150, 185, 130, 65, 34, 61, 140, 230, 42, 212, 141,
        134, 63, 76, 177, 140, 147, 14, 76,
    ];

    // Construct address struct (defaults to pubkey hash, cashaddr and main network)
    let address = Address {
        body: raw_address.to_vec(),
        ..Default::default()
    };

    // Encode address
    let address_str = address.encode().unwrap();

    // bitcoincash:qh3krj5607v3qlqh5c3wq3lrw3wnuxw0sp8dv0zugrrt5a3kj6ucysfz8kxwv2k53krr7n933jfsunqex2w82sl
    println!("{}", address_str);
}

```
