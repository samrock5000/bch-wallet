use bitcoin_hashes::{ripemd160, sha256, Hash};
use core::str::FromStr;
use electrum_client::bitcoin::bip32::{DerivationPath, ExtendedPrivKey};
use electrum_client::bitcoin::Network;
use secp256k1::Secp256k1;
use std::fs::File;
use std::io::Read;

use crate::encryption;
use crate::store::storage::KEY_PATH;

// use super::address::get_address;

pub static PURPOSE: u32 = 44;
pub static MAINNET: u32 = 0;
pub static TESTNET: u32 = 1;
//ONLY SUPPORT 1 ACCOUNT FOR NOW
pub static ACCOUNT: u32 = 0;

pub enum CoinType {
    Mainnet,
    Testnet,
}

pub enum Change {
    External,
    Inner,
}
impl Change {
    fn internal() -> u32 {
        1
    }
    fn external() -> u32 {
        0
    }
}
//TODO make hd key stragety
pub fn default_testnet_derivation() -> Result<DerivationPath, electrum_client::bitcoin::bip32::Error>
{
    DerivationPath::from_str(format!("m/{PURPOSE}'/{TESTNET}'/{ACCOUNT}'/0/0").as_str())
}

pub fn get_hd_node_from_db_seed(
    password: Option<&str>,
    network: Network,
) -> Result<ExtendedPrivKey, String> {
    if password.is_some() {
        match load_seed(password) {
            Ok(seed) => match ExtendedPrivKey::new_master(network, &seed) {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        match load_seed(None) {
            Ok(seed) => match ExtendedPrivKey::new_master(network, &seed) {
                Ok(x) => Ok(x),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn load_seed(password: Option<&str>) -> Result<Vec<u8>, String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let mnemonic_path = path.join("seed");
    let file = File::open(mnemonic_path);
    let mut buffer = vec![];

    if file.is_ok() {
        _ = file.unwrap().read_to_end(&mut buffer);
    } else {
        return Err(file.unwrap_err().to_string());
    }

    if password.is_some() {
        match bincode::deserialize(&buffer) {
            Ok(data) => match encryption::decrypt(data, password.unwrap().to_string().clone()) {
                Ok(words) => Ok(words),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        match bincode::deserialize(&buffer) {
            Ok(seed) => Ok(seed),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn derive_hd_path_private_key(
    path: DerivationPath,
    xpriv: ExtendedPrivKey,
) -> Result<[u8; 32], electrum_client::bitcoin::bip32::Error> {
    let secp = Secp256k1::new();
    // let addr = get_address(
    //     &xpriv
    //         .derive_priv(&secp, &path)?
    //         .private_key
    //         .public_key(&secp)
    //         .serialize(),
    //     bitcoincash_addr::Network::Test,
    // )
    // .unwrap()
    // .encode()
    // .unwrap();
    Ok(xpriv.derive_priv(&secp, &path)?.private_key.secret_bytes())
}

pub fn derive_hd_path_public_key(
    path: DerivationPath,
    xpriv: ExtendedPrivKey,
) -> Result<[u8; 33], electrum_client::bitcoin::bip32::Error> {
    let secp = Secp256k1::new();

    Ok(xpriv
        .derive_priv(&secp, &path)?
        .to_keypair(&secp)
        .public_key()
        .serialize())
}

pub fn derive_path_pubkeyhash(
    path: DerivationPath,
    xpriv: ExtendedPrivKey,
) -> Result<Vec<u8>, electrum_client::bitcoin::bip32::Error> {
    let secp = Secp256k1::new();

    let pubkey = xpriv
        .derive_priv(&secp, &path)?
        .private_key
        .public_key(&secp);
    let hash = ripemd160::Hash::hash(&sha256::Hash::hash(&pubkey.serialize()));
    match hex::decode(hash.to_string()) {
        Ok(scripthash) => Ok(scripthash),
        Err(_) => Err(electrum_client::bitcoin::bip32::Error::InvalidDerivationPathFormat),
    }
}
pub fn derive_path_account(
    // account: u32,
    coint_type: CoinType,
    xpriv: ExtendedPrivKey,
) -> Result<Vec<u8>, electrum_client::bitcoin::bip32::Error> {
    let secp = Secp256k1::new();

    let network = match coint_type {
        CoinType::Mainnet => 0,
        CoinType::Testnet => 1,
    };
    let path = DerivationPath::from_str(format!("m/{PURPOSE}'/{network}'/{ACCOUNT}'").as_str())?;
    let pubkey = xpriv
        .derive_priv(&secp, &path)?
        .private_key
        .public_key(&secp);
    let hash = ripemd160::Hash::hash(&sha256::Hash::hash(&pubkey.serialize()));
    match hex::decode(hash.to_string()) {
        Ok(scripthash) => Ok(scripthash),
        Err(_) => Err(electrum_client::bitcoin::bip32::Error::InvalidDerivationPathFormat),
    }
}

pub fn derive_path_change(
    index: u32,
    change: Change,
    // account: u32,
    coint_type: CoinType,
    xpriv: ExtendedPrivKey,
) -> Result<Vec<u8>, electrum_client::bitcoin::bip32::Error> {
    let secp = Secp256k1::new();
    let network = match coint_type {
        CoinType::Mainnet => 0,
        CoinType::Testnet => 1,
    };

    let mut change_index = match change {
        Change::Inner => Change::internal(),
        Change::External => Change::external(),
    };

    let path = DerivationPath::from_str(
        format!("m/{PURPOSE}'/{network}/{ACCOUNT}'/{change_index}/{index}").as_str(),
    )?;
    let pubkey = xpriv
        .derive_priv(&secp, &path)?
        .private_key
        .public_key(&secp);
    let hash = ripemd160::Hash::hash(&sha256::Hash::hash(&pubkey.serialize()));
    match hex::decode(hash.to_string()) {
        Ok(scripthash) => Ok(scripthash),
        Err(_) => Err(electrum_client::bitcoin::bip32::Error::InvalidChildNumberFormat),
    }
}
/*
mod test {
    use super::*;
    use bitcoincash_addr::{Address, Network};
    use core::str::FromStr;
    use electrum_client::bitcoin::bip32::{self, DerivationPath};
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
    fn mtch_indx() {
        let PRIVATE_KEY_B58:&str = "tprv8ZgxMBicQKsPekbmKrUZW6eHQoNq3rrYt9Wee93eD2M5wQcvhmp9gyneGiQWuvV33NDko7eQzhSskq4nX3uzKmg9TxCyHptVuWHYua5f3aw";
        let xpriv =
            // ExtendedPrivKey::decode(&base58::decode(PRIVATE_KEY_B58).unwrap().as_slice()[0..78]);
            ExtendedPrivKey::from_str(PRIVATE_KEY_B58).unwrap();
        let path = DerivationPath::from_str("m/0'/1'/1589'").unwrap();

        let x = derive_path_change(0, Change::Inner, CoinType::Testnet, xpriv).unwrap();
        // let x = derive_path_scripthash(path, xpriv).unwrap();
        let address = Address {
            body: x.to_vec(),
            network: Network::Test,
            ..Default::default()
        };
        println!("{:?}", address.encode().unwrap());
        // bip32::
    }
}
*/
