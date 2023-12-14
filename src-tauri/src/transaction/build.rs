// use std::clone;

// use crate::address::address_to_pubkey_hash;

use crate::coins::selection::{
    non_token_amount_from_utxo, selection_final_candidates, BranchAndBoundCoinSelection,
    CoinSelectionAlgorithm, Excess, FeeRate, UtxoCandidates, WeightedUtxo,
};
use crate::coins::utxo::UnspentUtxos;
use crate::error::WalletError;
use crate::keys::bip44::{
    default_testnet_derivation, derive_hd_path_private_key, get_hd_node_from_db_seed,
};

use bitcoinsuite_core::ser::CompactUint;
use bitcoinsuite_core::tx::{CashToken, Commitment, NonFungibleTokenCapability, TxId, NFT};
use std::str::FromStr;
// use bitcoinsuite_core::tx::CashToken;
use bitcoinsuite_core::{
    hash::{Hashed, Sha256d},
    script::{PubKey, Script, ScriptMut},
    ser::BitcoinSer,
    tx::{Input, Output, Transaction},
};

use electrum_client::bitcoin::bip32::DerivationPath;
// use bytes::Bytes;
use secp256k1_abc::{Message, PublicKey, Secp256k1, SecretKey};

// use serde_json::Value;
use sigser::sighashtype::SigHashType;
use sigser::sigser::signature_ser;

#[derive(Clone, Debug)]
pub struct TokenOptions {
    //If category is none, it's a token genesis
    pub category: Option<TxId>,
    pub amount: CompactUint,
    pub nft: Option<NFT>,
    pub available_amount: u64,
}

fn calculate_dust(output: &Output) -> u64 {
    output.ser_len() as u64 * 3 + 444 as u64
}

pub fn create_nft(commitment: &str, capability: &str) -> Option<NFT> {
    let commitment = match hex::decode(hex::encode(commitment.as_bytes())) {
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
        println!("CREATE NFT {:#?} ", res);
        res
    } else {
        None
    }
}

fn get_private_key(derivation_path: &str) -> Result<[u8; 32], String> {
    match DerivationPath::from_str(derivation_path) {
        Ok(deriv_path) => {
            match get_hd_node_from_db_seed(None, electrum_client::bitcoin::Network::Testnet) {
                Ok(xp) => match derive_hd_path_private_key(deriv_path, xp) {
                    Ok(k) => Ok(k),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub type RawTransactionHex = String;
#[derive(Debug)]
pub struct DustAndRawTransactionHex {
    pub raw_tx: RawTransactionHex,
    pub dust: u64,
}

pub fn create_tx_for_destination_output(
    derivation_path: &str,
    token_options: Option<TokenOptions>,
    destination_script: &Script,
    src_script: &Script,
    amount: u64,
    utxos: UnspentUtxos,
    required_utxos: Option<UnspentUtxos>,
) -> Result<DustAndRawTransactionHex, WalletError> {
    // println!("TOKEN OPTIONS{:#?}", token_options);
    // println!("REQ TOKEN {:#?}", required_utxos);
    let fee = FeeRate::from_sat_per_vb(0.0);
    let mut w_utxos: Vec<WeightedUtxo> = Vec::new();
    let mut token_genesis_utxos: Vec<WeightedUtxo> = Vec::new();
    let mut token_spend_utxos: Vec<WeightedUtxo> = Vec::new();

    if let Some(r_utxos) = required_utxos.as_ref() {
        r_utxos.non_token.iter().for_each(|utxo| {
            let wtxo = WeightedUtxo {
                satisfaction_weight: fee.0 as usize,
                utxo: utxo.0.clone(),
            };
            token_genesis_utxos.push(wtxo);
        });
    }
    if let Some(r_utxos) = required_utxos.as_ref() {
        r_utxos.with_token.iter().for_each(|utxo| {
            let wtxo = WeightedUtxo {
                satisfaction_weight: fee.0 as usize,
                utxo: utxo.0.clone(),
            };
            token_spend_utxos.push(wtxo);
        });
    }
    utxos.non_token.iter().for_each(|utxo| {
        let wtxo = WeightedUtxo {
            satisfaction_weight: fee.0 as usize,
            utxo: utxo.0.clone(),
        };
        w_utxos.push(wtxo);
    });

    let cashtoken = if let Some(token) = token_options.as_ref() {
        let category = token.category.unwrap();
        let nft = token.nft.clone();
        Some(CashToken {
            amount: CompactUint(token.amount.0),
            category,
            nft,
        })
    } else {
        None
    };

    let token_change = if token_options.is_some() && !token_spend_utxos.is_empty() {
        let total_av_token_amt = token_options.as_ref().unwrap().available_amount;
        let dest_amount = token_options.as_ref().unwrap().amount.0;
        if dest_amount > total_av_token_amt {
            return Err(WalletError::Generic {
                reason: "Token: request amount > available".to_string(),
            });
        } else if total_av_token_amt == dest_amount {
            None
        } else {
            Some(CashToken {
                amount: CompactUint(total_av_token_amt - dest_amount),
                category: token_options.as_ref().unwrap().category.unwrap(),
                nft: match token_options.clone().unwrap().nft {
                    Some(nft) => Some(NFT {
                        capability: nft.capability,
                        commitment: nft.commitment,
                    }),
                    None => None,
                },
            })
        }
    } else {
        None
    };

    let mut destination_output = Output {
        script: destination_script.clone(),
        token: cashtoken.clone(),
        value: amount as u64,
    };
    let dust = calculate_dust(&destination_output);
    if amount < dust {
        return Err(WalletError::DustValue { amount, dust });
    }

    let available_sats = non_token_amount_from_utxo(&utxos);

    let raw_transactopn_hex: Result<String, WalletError> = if available_sats == amount {
        destination_output.value = amount;
        let av_utxos = utxos.non_token.iter().map(|x| x.0.clone()).collect();
        let mut utxos = UtxoCandidates {
            change: None,
            selected: av_utxos,
        };
        let tx_size = build_transaction_p2pkh(
            derivation_path,
            &mut utxos,
            vec![destination_output.clone()],
        );
        destination_output.value = destination_output.value - tx_size?.len() as u64 / 2;

        let tx_hex = build_transaction_p2pkh(derivation_path, &mut utxos, vec![destination_output]);
        Ok(tx_hex?)
    } else {
        let req_utxos = if token_genesis_utxos.is_empty() {
            token_spend_utxos
        } else {
            token_genesis_utxos
        };

        let mut maybe_change = Output {
            script: src_script.clone(),
            token: token_change,
            ..Default::default()
        };

        let coins = match BranchAndBoundCoinSelection::default().coin_select(
            req_utxos.clone(), //required utxos
            w_utxos,           //optional utxos
            FeeRate::from_sat_per_vb(0.0),
            amount,
            &destination_output,
        ) {
            Ok(selection) => {
                match selection.excess {
                    Excess::Change { amount, fee: _ } => {
                        let tx_size = build_transaction_p2pkh(
                            derivation_path,
                            &mut selection_final_candidates(&selection).unwrap(),
                            vec![maybe_change.clone(), destination_output.clone()],
                        )
                        .unwrap()
                        .len()
                            / 2;
                        let change_amount = amount;
                        let total_relay_fee = tx_size as u64;
                        //Check if change can cover relay fee and leftover is not below dust
                        if (change_amount - total_relay_fee) > calculate_dust(&maybe_change) {
                            maybe_change.value += change_amount;
                            maybe_change.value -= total_relay_fee;
                            build_transaction_p2pkh(
                                derivation_path,
                                &mut selection_final_candidates(&selection).unwrap(),
                                vec![maybe_change, destination_output],
                            )
                        } else {
                            let tx_size = build_transaction_p2pkh(
                                derivation_path,
                                &mut selection_final_candidates(&selection).unwrap(),
                                vec![destination_output.clone()],
                            )
                            .unwrap()
                            .len()
                                / 2;
                            destination_output.value -= tx_size as u64;
                            build_transaction_p2pkh(
                                derivation_path,
                                &mut selection_final_candidates(&selection).unwrap(),
                                vec![destination_output],
                            )
                        }
                    }
                    Excess::NoChange {
                        dust_threshold: _,
                        remaining_amount,
                        change_fee: _,
                    } => {
                        let tx_size = build_transaction_p2pkh(
                            derivation_path,
                            &mut selection_final_candidates(&selection).unwrap(),
                            vec![destination_output.clone()],
                        )
                        .unwrap()
                        .len()
                            / 2;
                        destination_output.value += remaining_amount;
                        destination_output.value -= tx_size as u64;
                        build_transaction_p2pkh(
                            derivation_path,
                            &mut selection_final_candidates(&selection).unwrap(),
                            vec![destination_output],
                        )
                    }
                }
            }
            Err(e) => Err(WalletError::Generic {
                reason: format!("Coin Selection Error: {:?}", e),
            }),
        };
        coins
    };
    match raw_transactopn_hex {
        Ok(tx_hex) => Ok(DustAndRawTransactionHex {
            dust,
            raw_tx: tx_hex,
        }),
        Err(e) => Err(WalletError::Generic {
            reason: e.to_string(),
        }),
    }
}

pub fn build_transaction_p2pkh(
    derivation_path: &str,
    selected_outputs: &mut UtxoCandidates,
    destination_outputs: Vec<Output>,
) -> Result<RawTransactionHex, WalletError> {
    let mut input_vec: Vec<Input> = Vec::new();
    let mut signed_inputs: Vec<Input> = Vec::new();
    let mut source_outputs: Vec<Output> = Vec::new();
    let mut input_index = 0;

    for input in selected_outputs.selected.iter() {
        let input_unsigned = Input {
            prev_out: input.outpoint,
            script: Script::default(),
            sequence: 0,
        };

        input_vec.push(input_unsigned);

        source_outputs.push(input.output.clone());
    }

    let mut tx_unsigned = Transaction {
        version: 2,
        inputs: input_vec,
        outputs: destination_outputs.clone(),
        locktime: 0,
    };

    for input in selected_outputs.selected.iter() {
        let signature_serialized = signature_ser(
            input_index as u32,
            &source_outputs,
            &mut tx_unsigned,
            &SigHashType::ALL_BIP143_UTXOS,
        );
        if get_private_key(derivation_path).is_err() {
            return Err(WalletError::Generic {
                reason: get_private_key(derivation_path).unwrap_err().to_string(),
            });
        }

        let secret_key =
            SecretKey::from_slice(&mut get_private_key(derivation_path).clone().unwrap());

        let secp = Secp256k1::new();
        let sighash = hex::decode(signature_serialized).unwrap();
        let sighash = Sha256d::digest(sighash.clone());
        let msg = Message::from_slice(&sighash.to_le_bytes()).expect("Impossible");

        let sig = secp.schnorrabc_sign_no_aux_rand(&msg, &secret_key.unwrap());

        let pubkey = PublicKey::from_secret_key(&secp, &secret_key.unwrap());
        let hashtype = [SigHashType::ALL_BIP143_UTXOS.to_u32() as u8];
        let sig = vec![sig.as_ref().as_slice(), &hashtype].concat();
        let mut sig_script = ScriptMut::with_capacity(1 + 64 + 1 + PubKey::SIZE);
        sig_script.put_bytecode(&[sig.len() as u8]);
        sig_script.put_bytecode(&sig);
        sig_script.put_bytecode(&[PubKey::SIZE as u8]);
        sig_script.put_bytecode(hex::decode(pubkey.to_string()).unwrap().to_vec().as_ref());

        let signed_input = Input {
            prev_out: input.outpoint,
            script: sig_script.clone().freeze(),
            sequence: 0,
        };
        signed_inputs.push(signed_input);
        if input_index == tx_unsigned.inputs.len() {
            break;
        }
        input_index += 1;
    }

    let tx_signed = Transaction {
        version: 2,
        inputs: signed_inputs,
        outputs: destination_outputs,
        locktime: 0,
    };

    Ok(hex::encode(tx_signed.ser()))
}

/*

mod test {
    use bitcoinsuite_core::{
        ser::BitcoinSer,
        ser::CompactUint,
        tx::{CashToken, Commitment, NonFungibleTokenCapability, TxId, NFT},
    };
    use bytes::Bytes;

    /// let zero_tx_hash =
    ///     "cadef383d48ceaa0ee0af8f8d75b478f3bed6a5d1ef8b5be50022e056b975da8"
    ///         .parse::<TxId>()
    ///         .unwrap();

    #[test]
    fn token() {
        let category = "cadef383d48ceaa0ee0af8f8d75b478f3bed6a5d1ef8b5be50022e056b975da8"
            .parse::<TxId>()
            .unwrap();

        let token = CashToken {
            amount: CompactUint(11121),
            category,
            nft: Some(NFT {
                capability: NonFungibleTokenCapability(bitcoinsuite_core::tx::Capability::Minting),
                commitment: Commitment(Bytes::copy_from_slice(&[48, 48, 48])),
            }),
        };

        println!("{:?}", hex::decode(""));
    }
}
*/
