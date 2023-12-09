use bitcoincash_addr::{AddressCodec, CashAddrCodec, HashType, Network};

use crate::{
    error::WalletError,
    network::{self, electrum::get_unspent_utxos},
};
pub static KEY_PATH: &'static str = ".p2p-wallet/";

#[tauri::command]
pub fn store_utxos(address: String, data: String) -> Result<(), String> {
    match dirs::home_dir().is_some() {
        true => match sled::open(dirs::home_dir().unwrap().join(KEY_PATH)) {
            Ok(db) => {
                let address_content = CashAddrCodec::decode(address.as_str());
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
