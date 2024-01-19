use bitcoincash_addr::{AddressCodec, CashAddrCodec, HashType, Network};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletConfig {
    network: Value,
    network_url: Value,
}

use crate::{
    error::WalletError,
    network::{self, electrum::get_unspent_utxos},
    wallet,
};
pub static KEY_PATH: &'static str = ".cashcaster/";

#[tauri::command]
pub fn store_utxos(address: String, data: String) -> Result<(), String> {
    match dirs::home_dir().is_some() {
        true => match sled::open(dirs::home_dir().unwrap().join(KEY_PATH).join("utxos")) {
            Ok(db) => {
                let address_content = CashAddrCodec::decode(address.as_str());
                println!("HELLO ARE WE SAVING");
                let script_hash = if address_content.is_ok() {
                    address_content.unwrap().body
                } else {
                    return Err("Bad Address encoding".to_string());
                };
                let _ = db.insert(script_hash, data.as_bytes());
                println!("utxo store updated");

                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
        false => Err("dirs::home_dir() is None".to_string()),
    }
}
#[tauri::command]
pub fn store_config(address: String, wallet_conf: Value) -> Result<(), String> {
    match sled::open(dirs::home_dir().unwrap().join(KEY_PATH).join("walletconf")) {
        Ok(db) => match CashAddrCodec::decode(&address) {
            Ok(addr) => {
                _ = db.insert(addr.body, json!(wallet_conf).as_str().unwrap().as_bytes());
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
pub fn get_store_config(address: String) -> Result<Value, String> {
    match sled::open(dirs::home_dir().unwrap().join(KEY_PATH).join("walletconf")) {
        Ok(db) => match db.get(address) {
            Ok(data) => match data {
                Some(res) => Ok(Value::from(res.to_vec())),
                None => Ok(Value::default()),
            },
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}
