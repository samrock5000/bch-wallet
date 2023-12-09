use bitcoin_hashes::{ripemd160, sha256, Hash};
use bitcoincash_addr::{Address, Network};

pub fn get_address(
    pubkey: &[u8],
    network: Network,
) -> Result<bitcoincash_addr::Address, bitcoincash_addr::cashaddr::DecodingError> {
    let hash = sha256::Hash::hash(&pubkey);
    let hash = ripemd160::Hash::hash(&hash);
    let address = Address {
        body: hash.to_vec(),
        network,
        ..Default::default()
    };
    Ok(address)
}
