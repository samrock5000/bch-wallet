pub mod address;
pub mod bip44;

// pub mod error;
// mod pbkdf2;
/*
mod test {

    use bitcoin_hashes::{ripemd160, sha256, Hash};
    use bitcoincash_addr::{Address, Network};
    use core::str::FromStr;
    use electrum_client::bitcoin::base58;
    use electrum_client::bitcoin::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
    use electrum_client::bitcoin::secp256k1::Secp256k1;
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
    #[test]
    fn bip32_test() {
        let secp = Secp256k1::new();
        // let cn = ChildNumber::from_hardened_idx(42350).unwrap();
        let path = DerivationPath::from_str("m/0'/1'/1589'").unwrap();
        let PRIVATE_KEY_B58:&str = "tprv8ZgxMBicQKsPekbmKrUZW6eHQoNq3rrYt9Wee93eD2M5wQcvhmp9gyneGiQWuvV33NDko7eQzhSskq4nX3uzKmg9TxCyHptVuWHYua5f3aw";
        let x =
            ExtendedPrivKey::decode(&base58::decode(PRIVATE_KEY_B58).unwrap().as_slice()[0..78]);
        println!(
            "{:?}",
            x.unwrap()
                .derive_priv(&secp, &path)
                .unwrap()
                .private_key
                .secret_bytes()
        );
    }
}
*/
