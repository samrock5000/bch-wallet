use crate::address::address_to_pubkey_hash;
use crate::error::WalletError;

use crate::store::storage::KEY_PATH;
use bitcoinsuite_core::{
    hash::{Hashed, Sha256d, ShaRmd160},
    script::Script,
    ser::CompactUint,
    tx::{
        Capability, CashToken, Commitment, NonFungibleTokenCapability, OutPoint, Output, TxId, NFT,
    },
};
use bytes::Bytes;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Utxo {
    pub height: u32,
    pub output: Output,
    pub outpoint: OutPoint,
}

///Unspent Output "utxo" with token data
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnspentOutputWithTokens(pub Utxo);
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnspentOutput(pub Utxo);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnspentUtxos {
    pub with_token: Vec<UnspentOutputWithTokens>,
    pub non_token: Vec<UnspentOutput>,
}

pub fn get_utxos_for_address(address: &str) -> Result<UnspentUtxos, WalletError> {
    serde_json_to_utxo(get_db_utxo_unspent(address)?, address)
}

pub fn get_db_utxo_unspent(address: &str) -> Result<Value, WalletError> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let db = sled::open(path)?;
    if address_to_pubkey_hash(address).is_ok() {
        let pubkey_hash = address_to_pubkey_hash(address)?;
        let res = match db.get(pubkey_hash) {
            Ok(val) => match val {
                Some(utxo_bytes) => {
                    let utxo_serde_json =
                        serde_json::from_slice::<serde_json::Value>(utxo_bytes.to_vec().as_ref());
                    utxo_serde_json
                }
                None => Ok(Value::Array([].to_vec())),
            },
            Err(e) => Err(serde_json::Error::from(WalletError::Generic {
                reason: e.to_string(),
            })),
        };
        Ok(res?)
    } else {
        Err(WalletError::AddresssDecodeError {
            reason: address_to_pubkey_hash(address).unwrap_err().to_string(),
        })
    }
}

/// Converts serde json values to bitcoinsuite types and structures them to [UnspentUtxos]
pub fn serde_json_to_utxo(utxos: Value, src_addr: &str) -> Result<UnspentUtxos, WalletError> {
    let pubkey_hash = address_to_pubkey_hash(src_addr)?;
    let pubkey_hash = hex::encode::<&[u8]>(pubkey_hash.as_ref());
    let pubkey_hash = ShaRmd160::from_le_hex(&pubkey_hash)?;
    let mut utxo_vec: Vec<UnspentOutput> = vec![];
    let mut token_utxo_vec: Vec<UnspentOutputWithTokens> = vec![];

    let mut tokens_utxos = vec![];
    let mut utxos_no_tokens = vec![];

    for utxo in utxos.as_array().unwrap() {
        if !utxo["token_data"].is_null() {
            tokens_utxos.push(utxo.clone());
        };
    }

    for utxo in utxos.as_array().unwrap() {
        if utxo["token_data"].is_null() {
            utxos_no_tokens.push(utxo.clone());
        };
    }

    utxos_no_tokens.iter().for_each(|key| {
        let height = match key["height"].as_number() {
            Some(hgt) => match hgt.as_u64() {
                Some(height) => Ok(height),
                None => Err(WalletError::Generic {
                    reason: "key[height].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[height].as_number() ERR".to_string(),
            }),
        };

        let value = match key["value"].as_number() {
            Some(v) => match v.as_u64() {
                Some(value) => Ok(value),
                None => Err(WalletError::Generic {
                    reason: "key[value].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[value].as_number() ERR".to_string(),
            }),
        };

        let index = match key["tx_pos"].as_number() {
            Some(v) => match v.as_u64() {
                Some(index) => Ok(index),
                None => Err(WalletError::Generic {
                    reason: "key[index].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[index].as_number() ERR".to_string(),
            }),
        };

        let txid = match key["tx_hash"].as_str() {
            Some(index) => Ok(index),
            None => Err(WalletError::Generic {
                reason: "key[txid].as_str() ERR".to_string(),
            }),
        };
        if height.is_ok() && value.is_ok() && index.is_ok() && txid.is_ok() {
            let txid = Sha256d::from_be_hex(&txid.unwrap());

            let outpoint = OutPoint {
                outpoint_index: index.unwrap() as u32,
                txid: txid.unwrap().0.into(),
            };

            let output = Output {
                script: Script::p2pkh(&pubkey_hash),
                token: None,
                value: value.unwrap() as u64,
            };

            let utxo = Utxo {
                height: height.unwrap() as u32,
                outpoint,
                output,
            };
            let res = UnspentOutput(utxo);
            utxo_vec.push(res);
        }
    });

    tokens_utxos.iter().for_each(|key| {
        let height = match key["height"].as_number() {
            Some(hgt) => match hgt.as_u64() {
                Some(height) => Ok(height),
                None => Err(WalletError::Generic {
                    reason: "key[height].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[height].as_number() ERR".to_string(),
            }),
        };

        let value = match key["value"].as_number() {
            Some(v) => match v.as_u64() {
                Some(value) => Ok(value),
                None => Err(WalletError::Generic {
                    reason: "key[value].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[value].as_number() ERR".to_string(),
            }),
        };

        let index = match key["tx_pos"].as_number() {
            Some(v) => match v.as_u64() {
                Some(index) => Ok(index),
                None => Err(WalletError::Generic {
                    reason: "key[index].as_number() ERR".to_string(),
                }),
            },
            None => Err(WalletError::Generic {
                reason: "key[index].as_number() ERR".to_string(),
            }),
        };

        let txid = match key["tx_hash"].as_str() {
            Some(txid) => Ok(txid),
            None => Err(WalletError::Generic {
                reason: "key[txid].as_str() ERR".to_string(),
            }),
        };
        //token_data["amount"])
        let token = match key["token_data"].as_object() {
            Some(token_data) => {
                let amount = match token_data["amount"].as_str() {
                    Some(amt) => match amt.parse::<u64>() {
                        Ok(amt) => Ok(amt),
                        Err(e) => Err(WalletError::Generic {
                            reason: e.to_string(),
                        }),
                    },

                    None => Err(WalletError::Generic {
                        reason: "token_data[amt].as_str() ERR".to_string(),
                    }),
                };

                let category = match token_data["category"].as_str() {
                    Some(categoy) => Ok(categoy),
                    None => Err(WalletError::Generic {
                        reason: "token_data[categoy].as_str() ERR".to_string(),
                    }),
                };

                let nft: Result<Option<NFT>, WalletError> = if token_data.contains_key("nft") {
                    match token_data["nft"].as_object() {
                        Some(nft) => {
                            let commitment = match nft["commitment"].as_str() {
                                Some(commitment) => {
                                    Commitment(hex::decode(commitment).unwrap().into())
                                }
                                None => Commitment(Bytes::copy_from_slice(vec![].as_ref())),
                            };
                            let capability = match nft["capability"].as_str() {
                                Some(capability) => match capability {
                                    "none" => Ok(NonFungibleTokenCapability(Capability::None)),
                                    "mutable" => {
                                        Ok(NonFungibleTokenCapability(Capability::Mutable))
                                    }
                                    "minting" => {
                                        Ok(NonFungibleTokenCapability(Capability::Minting))
                                    }
                                    _ => Err(Value::Null),
                                },
                                None => Err(Value::Null),
                            };
                            let nft = if capability.is_ok() {
                                Ok(Some(NFT {
                                    capability: capability.unwrap(),
                                    commitment,
                                }))
                            } else {
                                Ok(None)
                            };

                            nft
                        }
                        None => Ok(None),
                    }
                } else {
                    Ok(None)
                };

                let cashtoken: Result<Option<CashToken>, WalletError> =
                    if category.is_ok() && amount.is_ok() && nft.is_ok() {
                        let category = Sha256d::from_be_hex(&category.unwrap());

                        let cashtoken = CashToken {
                            amount: CompactUint(amount.unwrap()),
                            category: TxId::from(category.unwrap()),
                            nft: nft.unwrap(),
                        };
                        Ok(Some(cashtoken))
                    } else {
                        Err(WalletError::Generic {
                            reason: "cashtoken serde json error".to_string(),
                        })
                    };
                cashtoken
            }
            None => Err(WalletError::Generic {
                reason: "key[token_data].as_object()".to_string(),
            }),
        };

        if height.is_ok() && value.is_ok() && index.is_ok() && txid.is_ok() && token.is_ok() {
            let txid = Sha256d::from_be_hex(&txid.unwrap());

            let outpoint = OutPoint {
                outpoint_index: index.unwrap() as u32,
                txid: txid.unwrap().0.into(),
            };

            let output = Output {
                script: Script::p2pkh(&pubkey_hash),
                token: token.unwrap(),
                value: value.unwrap() as u64,
            };

            let utxo = Utxo {
                height: height.unwrap() as u32,
                outpoint,
                output,
            };
            let res = UnspentOutputWithTokens(utxo);
            token_utxo_vec.push(res);
        }
    });

    Ok(UnspentUtxos {
        non_token: utxo_vec,
        with_token: token_utxo_vec,
    })
}

/*

mod test {
    // use std::{collections::BTreeMap, path::Path, str::FromStr};

    // use bip32::{ChildNumber, DerivationPath, ExtendedPrivateKey, PrivateKey};
    // use bitcoincash_addr::{AddressCodec, CashAddrCodec, HashType, Network};

    /*     use crate::{
        bip32::ExtendedPrivateKey,
        // bip44::private_hierarchy::{Account, Change, CoinType, Purpose},
        encryption::error::WrongPasswordError,
    }; */

    // use crate::keys::bip32::ExtendedPrivateKey;

    use super::*;

    /*     fn get_accounts_overview() -> String {
        let path = dirs::home_dir().unwrap().join(KEY_PATH);
        let db = sled::open(path).unwrap();
        if !db.contains_key("master_public_key").unwrap() {
            return "".to_string();
        }
        let master_pub_key = db.get("master_public_key").unwrap().clone();
        let master_pub_key = MasterPublicKey::load(&master_pub_key.unwrap()[..]);
        master_pub_key.jsonify()
    } */
    fn send_passphrase(passphrase: String) {
        let passphrase_encoded: Vec<u8> = bincode::serialize(&passphrase).unwrap();
        let path = dirs::home_dir().unwrap().join(KEY_PATH);
        let db = sled::open(path).unwrap();
        db.insert("passphrase".to_string(), passphrase_encoded);
    }

    #[test]
    fn test_db_utxo() {
        let sample_addr = "bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf";
        let sample_addr2 = "bchtest:pqaf54jyp7xlx5dah84gqnry6n786evj55zwnnalcq";
        // let x = get_db_utxo(sample_addr);
        //     from_serde_json(x.unwrap());
        // let hashtype = bitcoincash_addr::HashType::Key;
        // let network = Network::Test;
        // let x = CashAddrCodec::decode(sample_addr);
        // let y = CashAddrCodec::encode(&x.as_ref().unwrap().body, hashtype, network);
        // println!("{:?} {:?}", x, y);
        // let x = get_db_utxo_unspent(sample_addr2);
        let x = get_db_utxo_unspent(sample_addr);
        let y = serde_json_to_utxo(x.unwrap(), sample_addr);
        // println!("{:#?}", x);
        println!("{:#?}", y);
    }
    /*    #[test]
    fn coin() {
        let target_master_key = String::from("tprv8ZgxMBicQKsPekbmKrUZW6eHQoNq3rrYt9Wee93eD2M5wQcvhmp9gyneGiQWuvV33NDko7eQzhSskq4nX3uzKmg9TxCyHptVuWHYua5f3aw");
        let master_key =
            ExtendedPrivateKey::import_key_from_base58_check(target_master_key.as_str());
        let mut master_key = MasterPrivateKey::create_from_key(master_key.unwrap());
        let x = master_key.new_change_keypair(1, 1, Some(0));
        println!(" child_key_data {:?}", x);
    } */

    #[test]
    fn test_create_receive_addresses() {
        /*     // let path = dirs::home_dir().unwrap().join(KEY_PATH);
        // let db = sled::open(path.clone()).unwrap();
        let target_master_key = String::from("tprv8ZgxMBicQKsPekbmKrUZW6eHQoNq3rrYt9Wee93eD2M5wQcvhmp9gyneGiQWuvV33NDko7eQzhSskq4nX3uzKmg9TxCyHptVuWHYua5f3aw");
        // let seeds =
        // let p = DerivationPath::from_str(format!("m/0'/{addrs}").as_str());
        // PrivateKey::
        // let master_key = ExtendedPrivateKey::from_str(&target_master_key).unwrap(); //import_key_from_base58_check(target_master_key.as_str());

        let mut addrs: u32 = 0;
        while addrs < 100 {
            let xpubs = master_key
                .derive_child(ChildNumber::from(addrs))
                .unwrap()
                .public_key();
            println!(
                "{:?}",
                xpubs // .derive_public_key()
                      // .get_address()
                      // .unwrap()
                      // .encode()
                      // .unwrap()
            );
            // let receive_addrs_path = path.join("receive_addresses");
            // let db = sled::open(receive_addrs_path).unwrap();
            // let _ = db.insert(
            //     xpubs.clone().key_data,
            //     xpubs.get_address().unwrap().encode().unwrap().as_bytes(),
            // );
            addrs += 1;
        } */
    }
    /*  #[test]
    fn test_get_receive_addresses() {
        let path = dirs::home_dir().unwrap().join(KEY_PATH);
        let db = sled::open(path.clone()).unwrap();
        let master_pub_key = db.get("master_public_key").unwrap().clone();

        let receive_addrs_path = path.join("receive_addresses");
        let db = sled::open(receive_addrs_path).unwrap();

        let master_pub_key = MasterPublicKey::load(&master_pub_key.unwrap()[..]);
        println!("{:?}", master_pub_key);
        /*   let store_key = master_pub_key
            .public_key
            .derive_child_key(99)
            .unwrap()
            .key_data;
        let res = db.get(store_key).unwrap();
        println!(
            "{:?}",
            bitcoincash_addr::Address::decode(
                String::from_utf8(res.unwrap()[..].to_vec())
                    .unwrap()
                    .as_str()
            )
            .unwrap()
            .encode()
            .unwrap()
        ); */
    } */


    //Possibly useful later?
    fn calculate_dust(output: &str /* Output */) -> u64 {
    let output = Value::from(output);
    let amount = match output["amount"].as_number() {
        Some(v) => match v.as_u64() {
            Some(value) => value,
            None => 0, /*  Err(WalletError::Generic {
                           reason: "key[value].as_number() ERR".to_string(),
                       }), */
        },
        None => 0,
    };
    //Should be to unwrap, Validation should have happened.
    let script = address_to_p2pkh(output["script"].as_str().unwrap()).unwrap();
    let token = match output["token"].as_object() {
        Some(token_data) => {
            let amount = match token_data["amount"].as_str() {
                Some(amt) => match amt.parse::<u64>() {
                    Ok(amt) => Ok(amt),
                    Err(e) => Err(WalletError::Generic {
                        reason: e.to_string(),
                    }),
                },

                None => Err(WalletError::Generic {
                    reason: "token_data[amt].as_str() ERR".to_string(),
                }),
            };

            let category = match token_data["category"].as_str() {
                Some(categoy) => Ok(categoy),
                None => Err(WalletError::Generic {
                    reason: "token_data[categoy].as_str() ERR".to_string(),
                }),
            };

            let nft: Result<Option<NFT>, WalletError> = if token_data.contains_key("nft") {
                match token_data["nft"].as_object() {
                    Some(nft) => {
                        let commitment = match nft["commitment"].as_str() {
                            Some(commitment) => Commitment(hex::decode(commitment).unwrap().into()),
                            None => Commitment(Bytes::copy_from_slice(vec![].as_ref())),
                        };
                        let capability = match nft["capability"].as_str() {
                            Some(capability) => match capability {
                                "none" => Ok(NonFungibleTokenCapability(tx::Capability::None)),
                                "mutable" => {
                                    Ok(NonFungibleTokenCapability(tx::Capability::Mutable))
                                }
                                "minting" => {
                                    Ok(NonFungibleTokenCapability(tx::Capability::Minting))
                                }
                                _ => Err(Value::Null),
                            },
                            None => Err(Value::Null),
                        };
                        let nft = if capability.is_ok() {
                            Ok(Some(NFT {
                                capability: capability.unwrap(),
                                commitment,
                            }))
                        } else {
                            Ok(None)
                        };

                        nft
                    }
                    None => Ok(None),
                }
            } else {
                Ok(None)
            };

            let cashtoken: Result<Option<CashToken>, WalletError> =
                if category.is_ok() && amount.is_ok() && nft.is_ok() {
                    let category = Sha256d::from_be_hex(&category.unwrap());

                    let cashtoken = CashToken {
                        amount: CompactUint(amount.unwrap()),
                        category: TxId::from(category.unwrap()),
                        nft: nft.unwrap(),
                    };
                    Ok(Some(cashtoken))
                } else {
                    Err(WalletError::Generic {
                        reason: "cashtoken serde json error".to_string(),
                    })
                };
            cashtoken
        }
        None => Ok(None),
    };
    let output = Output {
        script,
        token: token.unwrap(),
        value: amount,
    };
    output.ser_len() as u64 * 3 + 444 as u64
}



}
*/
