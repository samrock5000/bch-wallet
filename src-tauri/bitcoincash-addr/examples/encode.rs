use bitcoincash_addr::Address;

fn main() {
    // Raw hash160 bytes
    let raw_address = [
        227, 97, 202, 154, 127, 153, 16, 124, 23, 166, 34, 224, 71, 227, 116, 93, 62, 25, 207, 128,
        78, 214, 60, 92, 64, 198, 186, 118, 54, 150, 185, 130, 65, 34, 61, 140, 230, 42, 212, 141,
        134, 63, 76, 177, 140, 147, 14, 76,
    ];

    // Construct address struct (defaults to pubkey hash, cash addr and main network)
    let address = Address {
        body: raw_address.to_vec(),
        ..Default::default()
    };

    // Encode address
    let address_str = address.encode().unwrap();

    // bitcoincash:qh3krj5607v3qlqh5c3wq3lrw3wnuxw0sp8dv0zugrrt5a3kj6ucysfz8kxwv2k53krr7n933jfsunqex2w82sl
    println!("{}", address_str);
}
