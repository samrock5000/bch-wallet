use crate::error::WalletError;
use bitcoincash_addr::{AddressCodec, CashAddrCodec};
use bitcoinsuite_core::{
    hash::{Hashed, ShaRmd160},
    script::Script,
};
/// 20 bit hash160 of public key
pub fn address_to_pubkey_hash(address: &str) -> Result<Vec<u8>, WalletError> {
    let address_content = CashAddrCodec::decode(address);
    let script_hash = if address_content.is_ok() {
        address_content.unwrap().body
    } else {
        vec![]
    };
    Ok(script_hash)
}
pub fn address_to_p2pkh(address: &str) -> Result<Script, WalletError> {
    let script_hash = if address_to_pubkey_hash(address).is_ok() {
        Ok(hex::encode(address_to_pubkey_hash(address)?))
    } else {
        Err(WalletError::AddresssDecodeError {
            reason: "invalid address".to_string(),
        })
    };

    let hash160 = &ShaRmd160::from_le_hex(&script_hash?)?;
    Ok(Script::p2pkh(hash160))
}
/*
mod test {
    use super::*;
    use bitcoinsuite_core::hash::Hashed;
    #[test]
    fn addr_to_lockscript() {
        const test_addr: &str = "bchtest:qptnz3u8atavszhaqk037v0fjrtahxmsl5mm45u3pf";
        let x = address_to_p2pkh(test_addr);
        assert_eq!(
            hex::encode(x.unwrap()),
            "76a91457314787eafac80afd059f1f31e990d7db9b70fd88ac"
        )
    }
}
*/
