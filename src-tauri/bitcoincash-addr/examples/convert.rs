use bitcoincash_addr::{Address, Network, Scheme};

fn main() {
    // Decode a base58 address
    let legacy_addr = "1NM2HFXin4cEQRBLjkNZAS98qLX9JKzjKn";
    let mut addr = Address::decode(legacy_addr).unwrap();

    // Change the address to a test net cashaddr
    addr.network = Network::Test;
    addr.scheme = Scheme::CashAddr;

    // Encode cashaddr
    let cashaddr_str = addr.encode().unwrap();

    // bchtest:qr4zgpuznfg923ntyauyeh5v7333v72xhum2dsdgfh
    println!("{}", cashaddr_str);
}
