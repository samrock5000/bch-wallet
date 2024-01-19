// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use cashcaster::address::address_to_p2pkh;
use cashcaster::coins::utxo::{
    get_db_utxo_unspent, get_utxos_for_address, serde_json_to_utxo, UnspentUtxos,
};
use cashcaster::keys::address::get_address;
use cashcaster::keys::bip44::{
    default_testnet_derivation, derive_hd_path_public_key, get_hd_node_from_db_seed,
};
// use bincode::ErrorKind;
use bitcoinsuite_core::hash::{Hashed, Ripemd160, Sha256, Sha256d};
use bitcoinsuite_core::script::{Op, Script};
use bitcoinsuite_core::ser::{BitcoinSer, CompactUint};
use bitcoinsuite_core::tx::{
    self, CashToken, Commitment, NonFungibleTokenCapability, Output, Transaction, TxId, NFT,
};
use bytes::Bytes;
use electrum_client::bitcoin::network::constants::ParseMagicError;
use num_bigint::{BigUint, ToBigUint};
use serde::{Deserialize, Deserializer, Serialize};
use std::fs::File;
use std::future::IntoFuture;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{utils::config::AppUrl, window::WindowBuilder, WindowUrl};
use tauri::{Manager, Window};
use url::Url;

use cashcaster::encryption;
// use cashcaster::keys::bip32::ExtendedPrivateKey;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use bitcoin_hashes::{ripemd160, /* sha256 */ Hash};
use bitcoincash_addr::{Address, AddressCodec, CashAddrCodec, HashType /* Network */};
use cashcaster::network::electrum::{
    get_address_history, get_mempool, get_unspent_utxos, get_utxos_balance_include_tokens,
    send_raw_transaction, subscribe,
};
use cashcaster::store::storage::{store_utxos, KEY_PATH};
use cashcaster::transaction::build::{
    build_transaction_p2pkh, create_tx_for_destination_output, RawTransactionHex, TokenOptions,
};
use cashcaster::{error::WalletError, network::electrum};
use electrum_client::bitcoin::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use electrum_client::bitcoin::{base58, Amount, Denomination, Network};
use electrum_client::bitcoin::{bip32, secp256k1};
use secp256k1::SecretKey;
use serde_json::{json, Value};
use sled::{self, Error};
use tauri::State;

/**
 * Network functions
*/

///get all the unspet utxos for this address
#[tauri::command]
async fn network_unspent_utxos(address: String, network_url: String) -> Result<String, String> {
    let res = electrum::get_unspent_utxos(address.as_str(), network_url.as_str());
    match res.await {
        Ok(unspent_utxos) => Ok(unspent_utxos),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn network_unspent_balance_no_tokens(
    address: String,
    network_url: String,
) -> Result<String, String> {
    let res = electrum::get_utxos_balance_no_tokens(address.as_str(), network_url.as_str());
    match res.await {
        Ok(balance) => Ok(balance),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn network_unspent_balance_include_tokens(
    address: String,
    network_url: String,
) -> Result<String, String> {
    let res = electrum::get_utxos_balance_include_tokens(address.as_str(), network_url.as_str());
    match res.await {
        Ok(balance) => Ok(balance),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn network_ping(network_url: String) -> Result<String, String> {
    let res = electrum::ping(network_url.as_str());
    match res.await {
        Ok(res) => Ok(res),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn address_history(
    address: String,
    network_url: String,
    // from_height: u32,
) -> Result<String, String> {
    let res = electrum::get_address_history(address.as_str(), network_url.as_str());
    match res.await {
        Ok(h) => Ok(h),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn subscribe_to_address(address: &str, network_url: &str) -> Result<String, String> {
    let res = subscribe(address, network_url);
    match res.await {
        Ok(hash) => Ok(hash),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn get_mempool_address(address: &str, network_url: &str) -> Result<String, String> {
    let res = get_mempool(address, network_url);
    match res.await {
        Ok(data) => Ok(data),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn unsubscribe_to_address(address: &str, network_url: &str) -> Result<bool, String> {
    use cashcaster::network::electrum::unsubscribe;
    let res = unsubscribe(address, network_url);
    match res.await {
        Ok(val) => Ok(val),
        Err(e) => Err(e.to_string()),
    }
}

/**
 * Key CRUD
 */
#[tauri::command]
fn generate_mnemonic() -> String {
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    mnemonic.phrase().to_string()
}
#[tauri::command]
fn valid_mnemonic(words: &str) -> Result<bool, String> {
    let x = Mnemonic::from_phrase(&words, Language::English);
    if x.is_ok() {
        Ok(true)
    } else {
        Err(x.unwrap_err().to_string())
    }
    // mnemonic.phrase().to_string()
}

#[tauri::command]
fn generate_seed(words: &str, salt: Option<&str>) -> Result<Vec<u8>, String> {
    let mnemonic = match Mnemonic::from_phrase(words, Language::English) {
        Ok(m) => Ok(m),
        Err(e) => Err(e.to_string()),
    };
    if mnemonic.is_ok() {
        if salt.is_none() {
            Ok(Seed::new(&mnemonic.unwrap(), "").as_bytes().to_vec())
        } else {
            Ok(Seed::new(&mnemonic.unwrap(), salt.unwrap())
                .as_bytes()
                .to_vec())
        }
    } else {
        Err(mnemonic.unwrap_err().to_string())
    }
}

#[tauri::command]
fn save_seed(seed: Vec<u8>, password: Option<&str>) -> Result<(), String> {
    if seed.len() != 64 {
        return Err(format!(
            "Invalid seed length: expected 64 found {}",
            seed.len()
        ));
    }
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let seed_path = path.join("seed");
    match save(&seed, &seed_path, password).is_ok() {
        true => {
            _ = save(&seed, &seed_path, password);
            Ok(())
        }
        false => Err(save(&seed, &seed_path, password).unwrap_err().to_string()),
    }
}

#[tauri::command]
fn load_mnemonic(password: Option<&str>) -> Result<String, String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let mnemonic_path = path.join("mnemonic");
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
                Ok(v) => match String::from_utf8(v) {
                    Ok(words) => Ok(words),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        match bincode::deserialize(&buffer) {
            Ok(data) => match String::from_utf8(data) {
                Ok(words) => Ok(words),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

#[tauri::command]
fn load_seed(password: Option<&str>) -> Result<Vec<u8>, String> {
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

#[tauri::command]
fn save_mnemonic(words: &str, password: Option<&str>) -> Result<(), String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let seed_path = path.join("mnemonic");
    let file = File::create(seed_path);
    let mnemonic = match Mnemonic::from_phrase(words, Language::English) {
        Ok(m) => Ok(m),
        Err(e) => Err(e.to_string()),
    };
    if mnemonic.is_ok() && password.is_none() {
        let mnemonic_bytes =
            bincode::serialize::<Vec<u8>>(&mnemonic.unwrap().phrase().as_bytes().to_vec());
        if mnemonic_bytes.is_err() {
            return Err(mnemonic_bytes.unwrap_err().to_string());
        }

        if file.is_err() {
            return Err(file.unwrap_err().to_string());
        } else {
            _ = file.unwrap().write_all(&mnemonic_bytes.unwrap());
        }
        Ok(())
    } else if mnemonic.is_ok() && password.is_some() {
        let bytes = encryption::encrypt(
            mnemonic.unwrap().phrase().as_bytes().to_vec(),
            password.unwrap().to_string(),
        );
        let mnemonic_bytes = bincode::serialize::<Vec<u8>>(&bytes);
        if mnemonic_bytes.is_err() {
            return Err(mnemonic_bytes.unwrap_err().to_string());
        }

        if file.is_err() {
            return Err(file.unwrap_err().to_string());
        } else {
            _ = file.unwrap().write_all(&mnemonic_bytes.unwrap());
        }

        Ok(())
    } else {
        Err(mnemonic.unwrap_err().into())
    }
}

fn save<P: AsRef<Path>>(
    secretkey: &Vec<u8>,
    path: &P,
    password: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();

    let master_key_encoded: Vec<u8> = bincode::serialize(&secretkey.as_slice())?;
    if password.is_some() {
        let master_key_encrypted =
            encryption::encrypt(master_key_encoded, password.unwrap().to_string());
        let mut file = File::create(path)?;
        file.write_all(&master_key_encrypted)?;

        Ok(())
    } else {
        let mut file = File::create(path)?;
        file.write_all(&master_key_encoded)?;

        Ok(())
    }
}

#[tauri::command]
fn valid_xpriv_base58_check(key: &str) -> Result<(), String> {
    match base58::decode_check(key) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn get_network(network: &str) -> Result<Network, String> {
    match network {
        "main" => Ok(Network::Bitcoin),
        "test" => Ok(Network::Testnet),
        _ => Err("Only main and test network are valid".to_string()),
    }
}

type Base58Xpriv = String;

#[tauri::command]
fn create_hd_node(seed: Vec<u8>, network: &str) -> Result<Base58Xpriv, String> {
    if seed.len() != 64 {
        return Err(format!(
            "Invalid seed length: expected 64 found {}",
            seed.len()
        ));
    }
    let network = match get_network(network) {
        Ok(netwrk) => Ok(netwrk),
        Err(e) => Err(e.to_string()),
    };
    if network.is_ok() {
        let xpriv = ExtendedPrivKey::new_master(network.unwrap(), &seed);
        match xpriv.is_ok() {
            true => Ok(base58::encode_check(&xpriv.unwrap().encode())),
            false => Err(xpriv.unwrap_err().to_string()),
        }
    } else {
        Err(network.unwrap_err().to_string())
    }
}

///Warning Overrides current key
#[tauri::command]
fn save_base58_xpriv(xpriv_base58: &str, password: Option<&str>) -> Result<(), String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_key_path = path.join("master_key");
    let privkey = match base58::decode_check(xpriv_base58) {
        Ok(val) => Ok(ExtendedPrivKey::decode(&val)),
        Err(e) => Err(e),
    };
    if privkey.is_ok() && password.is_some() {
        match privkey.unwrap() {
            Ok(key) => match save(&key.encode().to_vec(), &master_key_path, password).is_ok() {
                true => {
                    match save(&key.encode().to_vec(), &master_key_path, password) {
                        Ok(_) => print!("key saved"),
                        Err(_) => print!("key save fail"),
                    }
                    Ok(())
                }
                false => Err("create_master_key_from_import ERROR".to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        if privkey.is_ok() && password.is_none() {
            match privkey.unwrap() {
                Ok(key) => match save(&key.encode().to_vec(), &master_key_path, None).is_ok() {
                    true => {
                        match save(&key.encode().to_vec(), &master_key_path, password) {
                            Ok(_) => print!("key saved"),
                            Err(_) => print!("key save fail"),
                        }
                        Ok(())
                    }
                    false => Err("create_master_key_from_import ERROR1".to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        } else {
            Err(privkey.unwrap_err().to_string())
        }
    }
}

fn get_db_path_buf() -> Result<PathBuf, String> {
    match dirs::home_dir() {
        Some(p) => Ok(p.join(KEY_PATH)),
        None => Err("dirs::home_dir() error".to_string()),
    }
}

#[tauri::command]
fn address_from_hdpath(path: &str, network: &str) -> Result<String, String> {
    let hd_netwok = match network {
        "main" => Network::Bitcoin,
        _ => Network::Testnet,
    };
    let cash_addr_netwowk = match network {
        "main" => bitcoincash_addr::Network::Main,
        _ => bitcoincash_addr::Network::Test,
    };
    match DerivationPath::from_str(path) {
        //uses rust-bitcoin network enum
        Ok(p) => match get_hd_node_from_db_seed(None, hd_netwok) {
            Ok(key) => match derive_hd_path_public_key(p, key) {
                //uses bitcoincash_addr network enum. this is silly
                Ok(pubkey) => match get_address(&pubkey, cash_addr_netwowk) {
                    Ok(addr) => match addr.encode() {
                        Ok(bch_addr) => Ok(bch_addr),
                        Err(e) => Err(e.to_string()),
                    },
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn does_master_key_exist() -> bool {
    let p = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_key_path = p.join("master_key");
    dirs::home_dir().unwrap().join(master_key_path).exists()
}
#[tauri::command]
fn does_seed_exist() -> bool {
    let p = dirs::home_dir().unwrap().join(KEY_PATH);
    let seed_path = p.join("seed");
    dirs::home_dir().unwrap().join(seed_path).exists()
}

#[tauri::command]
fn does_wallet_exist() -> bool {
    let p = dirs::home_dir().unwrap().join(KEY_PATH);
    let mnemonic_path = p.join("mnemonic");
    dirs::home_dir().unwrap().join(mnemonic_path).exists()
}

#[tauri::command]
fn does_db_exist() -> bool {
    dirs::home_dir().unwrap().join(KEY_PATH).exists()
}

#[tauri::command]
fn create_db() {
    match sled::open(dirs::home_dir().unwrap().join(KEY_PATH)) {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn load_master_key<P: AsRef<Path>>(
    path: P,
    password: Option<&str>,
) -> Result<ExtendedPrivKey, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    if password.is_some() {
        let bytes: Vec<u8> = bincode::deserialize(&buffer)?;
        let buffer_decrypted = encryption::decrypt(bytes, password.unwrap().to_string().clone())?;
        let master_key: ExtendedPrivKey = bincode::deserialize(&buffer_decrypted[..])?;
        Ok(master_key)
    } else {
        let bytes: Vec<u8> = bincode::deserialize(&buffer)?;
        Ok(ExtendedPrivKey::decode(&bytes)?)
    }
}
#[tauri::command]
fn get_master_key<P: AsRef<Path>>(password: &str) {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_key_path = path.join("master_key");
    if password == "NONE" {}
}

//TODO implement in UI
#[tauri::command]
fn create_change_pubkeyhash_store(/* x_privkey: ExtendedPrivKey */) -> Result<(), String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_key_path = path.join("master_key");
    let secp = secp256k1::Secp256k1::new();
    let x_privkey = match load_master_key(master_key_path, None) {
        Ok(k) => Ok(k),
        Err(e) => Err(e.to_string()),
    };

    if x_privkey.is_ok() {
        let xtern_public_key = match DerivationPath::from_str(format!("m/44'/1'/0'/0").as_str()) {
            Ok(p) => match x_privkey.clone().unwrap().derive_priv(&secp, &p) {
                Ok(k) => Ok(bip32::ExtendedPubKey::from_priv(&secp, &k)),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };

        let intern_public_key = match DerivationPath::from_str(format!("m/44'/1'/0'/1").as_str()) {
            Ok(p) => match x_privkey.unwrap().derive_priv(&secp, &p) {
                Ok(k) => Ok(bip32::ExtendedPubKey::from_priv(&secp, &k)),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };

        let mut index = 0;
        let db = sled::open(path).unwrap();
        if intern_public_key.is_ok() && xtern_public_key.is_ok() {
            while index < 100 {
                let external_pubkey = xtern_public_key
                    .clone()
                    .unwrap()
                    .ckd_pub(&secp, ChildNumber::Normal { index })
                    .unwrap()
                    .public_key
                    .serialize();
                let hash = Sha256::digest(&external_pubkey);
                let hash = Ripemd160::digest(&hash);
                _ = db.insert(
                    format!("change-extern-{index}").to_string(),
                    hash.as_le_bytes(),
                );

                let internal_pubkey = intern_public_key
                    .clone()
                    .unwrap()
                    .ckd_pub(&secp, ChildNumber::Normal { index })
                    .unwrap()
                    .public_key
                    .serialize();
                let hash = Sha256::digest(&internal_pubkey);
                let hash = Ripemd160::digest(&hash);
                _ = db.insert(
                    format!("change-intern-{index}").to_string(),
                    hash.as_le_bytes(),
                );

                index += 1;
            }
        } else {
            return Err(intern_public_key.unwrap_err().to_string());
        }
    }
    Ok(())
}

//TODO will be used for hdkey stragety. need KV,
#[tauri::command]
fn get_pkh(key: &str) -> Result<String, String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let db = sled::open(path).unwrap();
    let x = db.get(key.as_bytes());
    Ok(hex::encode(x.unwrap().unwrap().to_vec()))
}

/**
 * Transaction creation
 */
#[tauri::command]
fn build_p2pkh_transaction(
    derivation_path: &str,
    destination_address: &str,
    source_address: &str,
    amount: u64,
    category: Option<&str>,
    token_amount: Option<&str>,
    commitment: Option<&str>,
    capability: Option<&str>,
    utxos: Value,
    required_utxos: Option<Value>,
) -> Result<RawTransactionHex, String> {
    println!("UTXOS JSON {:#?}", utxos);
    println!("REQUIRED JSON {:#?}", required_utxos);
    let mut token_available_amount = 0;

    if let Some(utxo) = &required_utxos {
        if let Some(txo) = utxo.as_array() {
            txo.iter().for_each(|utxo| {
                let amount = utxo["token_data"]["amount"].as_str();
                if let Some(amt) = amount {
                    if amt != "0" {
                        if let Some(a) = BigUint::parse_bytes(amt.as_bytes(), 10) {
                            token_available_amount = a
                                .to_u64_digits()
                                .first()
                                .map(|&digit| u64::from_le(digit))
                                .unwrap_or(0);
                        }
                    }
                }
            });
        }
    }
    let nft = if capability.is_some() && commitment.is_some() {
        match create_nft(commitment.unwrap(), capability.unwrap()) {
            Some(nft) => Some(nft),
            None => None,
        }
    } else {
        None
    };
    let available_utxos = match serde_json_to_utxo(utxos, source_address) {
        Ok(utxos) => Ok(utxos),
        Err(e) => Err(e.to_string()),
    };
    let req_utxos = match required_utxos {
        Some(data) => match serde_json_to_utxo(data, source_address) {
            Ok(utxos) => Some(utxos),
            _ => None,
        },
        None => None,
    };

    if validate_cash_address(destination_address).is_err()
        || validate_cash_address(source_address).is_err()
    {
        return Err("invalid cash address".to_string());
    }
    let destination_script = address_to_p2pkh(destination_address).unwrap();
    let src_script = address_to_p2pkh(source_address).unwrap();

    let raw_tx = if let Some(token_amount) = token_amount {
        let mut tok_amt = 0;
        let token_amount = BigUint::parse_bytes(token_amount.as_bytes(), 10);
        println!("{:?}", token_amount);
        if let Some(token_amount) = token_amount {
            let token_amount = token_amount.to_u64_digits();

            if nft.is_none() && token_amount.is_empty() {
                return Err(
                    "token amount must be greater than 0 for Fungible Token Types".to_string(),
                );
            }
            match token_amount.is_empty() {
                true => tok_amt = 0,
                false => tok_amt = u64::from_le(token_amount[0]),
            };
        }
        let token_data = create_token_options(category, tok_amt, nft, token_available_amount);

        match create_tx_for_destination_output(
            derivation_path,
            token_data,
            &destination_script,
            &src_script,
            amount,
            available_utxos.unwrap(),
            req_utxos,
        ) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
            //
        }
    } else {
        match create_tx_for_destination_output(
            derivation_path,
            None,
            &destination_script,
            &src_script,
            amount,
            available_utxos.unwrap(),
            req_utxos,
        ) {
            Ok(data) => Ok(data),
            Err(e) => Err(e.to_string()),
        }
    };
    println!("build p2pkh res\n{:?}\n", raw_tx);
    match raw_tx {
        Ok(res) => Ok(json!({"rawTx":res.raw_tx,"dust":res.dust}).to_string()),
        Err(e) => Err(e.to_string()),
    }
    // json!({})raw_tx
}
#[tauri::command]
async fn broadcast_transaction(transaction: &str, network_url: &str) -> Result<String, String> {
    match send_raw_transaction(transaction, network_url).await {
        Ok(txid) => Ok(txid.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
//TODO add source outputs to transaction. instead of showing tx hash and index. show actual
//source outputs
#[tauri::command]
async fn decode_transaction(transaction: &str) -> Result<Value, String> {
    let mut inputs = /* : Vec<Value> =  */Vec::new();
    let mut outputs = /* : Vec<Value> =  */Vec::new();

    let tx_hex = hex::decode(transaction);
    if tx_hex.is_err() {
        return Err(tx_hex.unwrap_err().to_string());
    }
    let txid = Value::String(Sha256d::digest(&tx_hex.as_ref().unwrap()).hex_be());
    let tx = Transaction::deser(&mut Bytes::from(tx_hex.unwrap()));
    if tx.is_err() {
        return Err(tx.unwrap_err().to_string());
    }
    tx.as_ref().unwrap().inputs.iter().for_each(|i| {
        let prevout = Value::String(hex::encode(Sha256d::from_be_bytes(
            i.prev_out.txid.as_bytes().to_owned(),
        )));
        let index = Value::Number(i.prev_out.outpoint_index.into());
        let res = json!({"prevout":prevout,"index":index});
        inputs.push(res)
    });

    tx.as_ref().unwrap().outputs.iter().for_each(|o| {
        let script = Value::String(hex::encode(o.script.ser()));
        let amount = Value::Number(o.value.into());

        let script = match script.as_str() {
            Some(s) => match lockscript_to_cash_address(s) {
                Ok(cashaddr) => Ok(cashaddr),
                Err(e) => Err(e.to_string()),
            },
            None => Err("bad script".into()),
        };
        let script = if script.is_ok() {
            script.unwrap()
        } else {
            return;
        };

        let token = if o.token.is_some() {
            let txid = hex::encode(o.token.clone().unwrap().category);

            match TxId::from_str(txid.as_str()) {
                Ok(txid) => {
                    let category = Value::String(hex::encode(txid.ser()));
                    let amount = Value::Number(o.token.as_ref().unwrap().amount.0.into());
                    if o.token.as_ref().unwrap().nft.is_some() {
                        let capability =
                            match o.token.as_ref().unwrap().nft.as_ref().unwrap().capability.0 {
                                bitcoinsuite_core::tx::Capability::Mutable => "mutable".to_string(),
                                bitcoinsuite_core::tx::Capability::Minting => "minting".to_string(),
                                bitcoinsuite_core::tx::Capability::None => "none".to_string(),
                            };

                        // let commitment =  hex::encode(o.token.as_ref().unwrap().commitment().0) ;
                        let commitment = o.token.as_ref().unwrap().commitment().0;
                            let commitment = hex::encode(commitment);
                        json!({"category":category,"amount":amount, "nft":{"capability": capability,"commitment":*commitment} })
                        // json!("")
                    } else {
                        json!({"category":category,"amount":amount, "nft":Value::Null})
                    }
                }
                Err(_) => Value::Null,
            }
        } else {
            Value::Null
        };
        let res = json!({"script":script,"amount":amount,"token":token});

        outputs.push(res)
    });

    Ok(json!({"inputs":inputs,"outputs":outputs,"txid":txid}))
}

//TODO ADD NETWORK ARG
// #[tauri::command]
fn lockscript_to_cash_address(script: &str) -> Result<String, String> {
    if script.len() == 0 {
        return Err("Invalid Script".to_string());
    }

    match hex::decode(&script.as_bytes()) {
        Ok(pkh) => {
            //decoded script includes variable length byte <1976>
            match CashAddrCodec::encode(
                &pkh[4..24],
                HashType::Key,
                bitcoincash_addr::Network::Test,
                false,
            ) {
                Ok(addr) => Ok(addr),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn update_utxo_store(address: &str, network_url: &str) -> Result<(), String> {
    match get_unspent_utxos(address, network_url).await {
        Ok(v) => {
            _ = store_utxos(address.to_string(), v.clone());
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn validate_cash_address(address: &str) -> Result<bool, String> {
    match CashAddrCodec::decode(address) {
        Ok(_) => Ok(true),
        Err(_) => Err(CashAddrCodec::decode(address).unwrap_err().to_string()),
    }
}

#[tauri::command]
fn validate_token_cash_address(address: &str) -> Result<u8, String> {
    match CashAddrCodec::decode(address) {
        Ok(data) => match data.token_support {
            true => Ok(1),
            false => Ok(0),
        },
        Err(_) => Err(CashAddrCodec::decode(address).unwrap_err().to_string()),
    }
}

#[tauri::command]
fn token_cash_address(address: &str) -> Result<String, String> {
    if CashAddrCodec::decode(address).is_ok() {
        let decoded = CashAddrCodec::decode(address).unwrap();
        match CashAddrCodec::encode(
            decoded.clone().as_body(),
            decoded.hash_type,
            decoded.network,
            true,
        ) {
            Ok(cashaddr) => Ok(cashaddr),
            Err(cashaddr) => Err(cashaddr.to_string()),
        }
    } else {
        Err(CashAddrCodec::decode(address).unwrap_err().to_string())
    }
}

#[tauri::command]
fn satoshi_to_bch(value: u64) -> Result<f64, String> {
    let res = Amount::from_sat(value);
    Ok(res.to_btc())
}
#[tauri::command]
fn bch_to_satoshi(value: f64) -> Result<u64, String> {
    match Amount::from_btc(value) {
        Ok(v) => Ok(v.to_sat()),
        Err(e) => Err(e.to_string()),
    }
}
//TODO remove can get sum from in memory db
#[tauri::command]
fn non_token_utxo_balance_db(address: &str) -> Result<u64, u64> {
    let utxo_from_db = get_db_utxo_unspent(&address);
    if utxo_from_db.is_ok() {
        let utxo_data = serde_json_to_utxo(utxo_from_db.unwrap(), address);
        let mut sum = 0;
        utxo_data
            .unwrap()
            .non_token
            .iter()
            .for_each(|utxo| sum += utxo.0.output.value);
        Ok(sum)
    } else {
        Ok(0)
    }
}
//TODO remove can get sum from in memory db
#[tauri::command]
fn utxo_balance_with_tokens_db(address: &str) -> Result<u64, u64> {
    let utxo_from_db = get_db_utxo_unspent(&address);
    if utxo_from_db.is_ok() {
        let utxo_data = serde_json_to_utxo(utxo_from_db.unwrap(), address);
        let mut sum = 0;
        utxo_data
            .as_ref()
            .unwrap()
            .with_token
            .iter()
            .for_each(|utxo| sum += utxo.0.output.value);

        Ok(sum)
    } else {
        Ok(0)
    }
}

#[tauri::command]
fn get_db_unspent_utxos(address: &str) -> Result<Value, String> {
    match get_db_utxo_unspent(&address) {
        Ok(utxos) => Ok(utxos),
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
fn get_non_token_utxo_data(address: &str) -> Result<Vec<Value>, u64> {
    let mut non_token_utxos = vec![];

    let res = if get_db_utxo_unspent(&address).is_ok() {
        let utxos = get_db_utxo_unspent(&address).unwrap();

        for utxo in utxos.as_array().unwrap() {
            if utxo["token_data"].is_null() {
                non_token_utxos.push(utxo.clone());
            };
        }
        Ok(non_token_utxos)
    } else {
        Ok(non_token_utxos)
    };
    // utxo_from_db
    res
}

#[tauri::command]
fn get_token_utxo_data(address: &str) -> Result<Vec<Value>, u64> {
    let mut tokens = vec![];

    let res = if get_db_utxo_unspent(&address).is_ok() {
        let utxos = get_db_utxo_unspent(&address).unwrap();

        for utxo in utxos.as_array().unwrap() {
            if !utxo["token_data"].is_null() {
                tokens.push(utxo.clone());
            };
        }
        Ok(tokens)
    } else {
        Ok(tokens)
    };
    // utxo_from_db
    res
}

pub fn create_nft(commitment: &str, capability: &str) -> Option<NFT> {
    // let commitment = match hex::decode(hex::encode(commitment.as_bytes())) {
    let commitment = match hex::decode(commitment) {
        Ok(hex) => Ok(hex),
        Err(e) => Err(e.to_string()),
    };
    let capability = match capability {
        "none" => NonFungibleTokenCapability(bitcoinsuite_core::tx::Capability::None),
        "minting" => NonFungibleTokenCapability(bitcoinsuite_core::tx::Capability::Minting),
        "mutable" => NonFungibleTokenCapability(bitcoinsuite_core::tx::Capability::Mutable),
        _ => NonFungibleTokenCapability(bitcoinsuite_core::tx::Capability::None),
    };
    if commitment.is_ok() {
        let res = Some(NFT {
            commitment: Commitment(commitment.unwrap().into()),
            capability,
        });
        res
    } else {
        None
    }
}

fn create_token_options(
    category: Option<&str>,
    amount: u64,
    nft: Option<NFT>,
    available_token_amount: u64,
) -> Option<TokenOptions> {
    let category: Option<TxId> = if category.is_some() {
        match Sha256d::from_be_hex(&category.unwrap()) {
            Ok(txid) => Some(txid.into()),
            Err(_) => None,
        }
    } else {
        None
    };

    let res = match nft {
        Some(nft) => TokenOptions {
            category,
            amount: CompactUint(amount),
            nft: Some(nft),
            available_amount: available_token_amount,
        },
        None => TokenOptions {
            category,
            amount: CompactUint(amount),
            nft: None,
            available_amount: available_token_amount,
        },
    };
    Some(res)
}

#[tauri::command]
fn valid_token_amount(amount: &str) -> Result<u8, String> {
    let amount = BigUint::parse_bytes(amount.as_bytes(), 10);
    let max_token_amount = BigUint::parse_bytes(b"9223372036854775807", 10);

    if amount.is_none() {
        return Err(format!("BigUint::parse_bytes Error: amount {:?}", amount));
    }

    /*if amount.as_ref().unwrap() == &BigUint::parse_bytes(b"0", 10).unwrap() {
        return Err("fungible token amount must be greater than 0".to_string());
    }*/

    if amount.unwrap() > max_token_amount.unwrap() {
        return Err("Invalid token prefix: exceeds maximum fungible token amount".to_string());
    }

    Ok(1)
}
#[tauri::command]
fn valid_nft(commitment: &str, capability: &str) -> Result<u8, String> {
    match create_nft(commitment, capability) {
        Some(_) => Ok(1),
        None => Err("invalid nft".to_string()),
    }
}

#[tauri::command]
async fn close_splash(window: Window) {
    // Close window
    window
        .get_window("splash")
        .expect("no window labeled 'splash' found")
        .close()
        .unwrap();
    println!("Done initializing.");
    // Show main window
    window
        .get_window("main")
        .expect("no window labeled 'main' found")
        .show()
        .unwrap();
}

#[tauri::command]
fn check_url(url: &str) -> Result<(), String> {
    match Url::parse(url) {
        Ok(res) => match res.scheme() {
            "tcp" => Ok(()),
            _ => Err(format!("invalid scheme {}", res.scheme())),
        },
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
fn utxo_cache(state: tauri::State<MemStore>) -> Result<Value, String> {
    Ok(state.inner().utxos.clone())
}

#[tauri::command]
fn address_cache(state: tauri::State<MemStore>) -> Result<Value, String> {
    Ok(state.inner().address.clone())
}
#[tauri::command]
fn wallet_exist(state: tauri::State<WalletExist>) -> Result<Value, String> {
    Ok(state.db.lock().unwrap().as_bool().into())
}
#[tauri::command]
fn wallet_exist_update(val: Value, state: tauri::State<WalletExist>) {
    *state.db.lock().unwrap() = val;
}
#[tauri::command]
fn wallet_cache(state: tauri::State<WalletData>) -> Result<Value, String> {
    // println!("wallet_cache {:#?}", state);
    Ok(json!(*state))
}
#[tauri::command]
fn update_bip44_path(val: Value, state: tauri::State<WalletData>) -> Result<Value, String> {
    state.db.lock().unwrap().bip44_path = val;
    Ok(json!(*state))
}
// #[tauri::command]
// fn wallet_data_update(val: Value, state: tauri::State<WalletData>) {
//     println!("WHATS THIS {:#?}", val);
//     *state.db.lock().unwrap() = val
// }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletExist {
    db: Mutex<Value>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WalletData {
    db: Mutex<MemStore>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MemStore {
    master_key: Value,
    balance: Value,
    token_satoshi_balance: Value,
    address: Value,
    network: Value,
    network_url: Value,
    mnemonic: Value,
    bip44_path: Value,
    utxos: Value,
    token_utxos: Value,
}

#[tokio::main]
async fn main() {
    const DERIVATION_PATH: &str = "m/44'/145'/0'/0/0";
    // const NETWORKURL: &str = "chipnet.imaginary.cash";
    const NETWORKURL: &str = "localhost";
    // tauri::async_runtime::spawn(start_server());
    let mut master_key = Value::default();
    let mut balance = Value::default();
    let mut token_satoshi_balance = Value::default();
    let mut address = Value::default();
    let mut network = Value::default();
    let mut network_url: Value = NETWORKURL.into();
    // let mut network_connection = 0;
    let mut mnemonic = Value::default();
    let mut bip44_path: Value = DERIVATION_PATH.into();
    let mut utxos = Value::default();
    let mut token_utxos = Value::default();
    let mut wallet_available = Value::default();

    // let connection_ack = network_ping(NETWORKURL.to_string())
    //     .into_future()
    //     .await
    //     .is_ok();
    // network_connection = if connection_ack { 1 } else { 0 };

    match does_wallet_exist() {
        true => {
            wallet_available = true.into();
            address = address_from_hdpath(DERIVATION_PATH, "test").unwrap().into();
            utxos = match get_non_token_utxo_data(address.as_str().unwrap()) {
                Ok(data) => {
                    let mut sum = 0;
                    serde_json_to_utxo(data.clone().into(), address.as_str().unwrap())
                        .unwrap()
                        .non_token
                        .iter()
                        .for_each(|utxo| sum += utxo.0.output.value);
                    balance = sum.into();
                    data.into()
                }
                Err(_) => Value::default(),
            };

            token_utxos = match get_token_utxo_data(address.as_str().unwrap()) {
                Ok(data) => {
                    let mut sum = 0;
                    serde_json_to_utxo(data.clone().into(), address.as_str().unwrap())
                        .unwrap()
                        .with_token
                        .iter()
                        .for_each(|utxo| sum += utxo.0.output.value);
                    token_satoshi_balance = sum.into();
                    data.into()
                }
                Err(_) => Value::default(),
            };
        }
        false => {
            wallet_available = false.into();
            match does_db_exist() {
                true => {}
                false => {
                    // wallet_available = false.into();
                    println!("Database created");
                    create_db()
                }
            }
        }
    }
    // println!("TOKEN UTXOS {:#?} {:#?}", utxos, token_utxos);
    let mem_store = MemStore {
        address,
        balance,
        bip44_path,
        master_key,
        mnemonic,
        network,
        network_url,
        token_satoshi_balance,
        token_utxos,
        utxos,
    };
    tauri::Builder::default()
        .manage(WalletData {
            db: Mutex::from(mem_store),
        })
        // .manage(MemStore {
        //     master_key,
        //     balance,
        //     token_satoshi_balance,
        //     address,
        //     network: "test".into(),
        //     network_url: NETWORKURL.into(),
        //     mnemonic,
        //     bip44_path: DERIVATION_PATH.into(),
        //     utxos,
        //     token_utxos,
        // })
        .manage(WalletExist {
            db: wallet_available.into(),
        })
        .plugin(tauri_plugin_websocket::init()) // Find way to manage web socket on backend?
        .invoke_handler(tauri::generate_handler![
            // wallet_data_update,
            update_bip44_path,
            wallet_exist_update,
            wallet_cache,
            wallet_exist,
            address_cache,
            utxo_cache,
            check_url,
            create_db,
            does_db_exist,
            get_db_unspent_utxos, // Maybe dont expose this to client
            validate_token_cash_address,
            token_cash_address,
            get_token_utxo_data,
            valid_nft,
            valid_token_amount,
            bch_to_satoshi,
            satoshi_to_bch,
            address_from_hdpath,
            load_seed,
            does_wallet_exist,
            load_mnemonic,
            save_mnemonic,
            does_seed_exist,
            save_seed,
            create_hd_node,
            generate_mnemonic,
            generate_seed,
            valid_mnemonic,
            // get_pkh, //TODO wait for HD key stragety
            valid_xpriv_base58_check,
            create_change_pubkeyhash_store,
            save_base58_xpriv,
            does_master_key_exist,
            validate_cash_address,
            address_history,
            broadcast_transaction,
            build_p2pkh_transaction,
            network_unspent_utxos,
            non_token_utxo_balance_db,
            utxo_balance_with_tokens_db,
            subscribe_to_address,
            unsubscribe_to_address,
            get_mempool_address,
            cashcaster::store::storage::store_utxos, /* db_utxos */
            update_utxo_store,
            network_unspent_balance_no_tokens,
            network_unspent_balance_include_tokens,
            network_ping,
            decode_transaction,
        ])
        .setup(|cashcaster| {
            let splashscreen_window = cashcaster.get_window("splash").unwrap();
            let main_window = cashcaster.get_window("main").unwrap();
            _ = main_window.hide();
            _ = splashscreen_window.show();
            // we perform the initialization code on a new task so the cashcaster doesn't freeze
            tauri::async_runtime::spawn(async move {
                // initialize your cashcaster here instead of sleeping :)
                println!("Initializing...");
                std::thread::sleep(std::time::Duration::from_secs(3));
                // println!("Done initializing.");
                // After it's done, close the splashscreen and display the main window
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri cashcasterlication");
}
