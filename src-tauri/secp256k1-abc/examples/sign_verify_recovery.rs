
extern crate bitcoin_hashes;
extern crate secp256k1_abc;

use bitcoin_hashes::{sha256, Hash};
use secp256k1_abc::recovery::{RecoverableSignature, RecoveryId};
use secp256k1_abc::{Error, Message, PublicKey, Secp256k1, SecretKey, Signing, Verification};

fn recover<C: Verification>(secp: &Secp256k1<C>,msg: &[u8],sig: [u8; 64],recovery_id: u8) -> Result<PublicKey, Error> {
    let msg = sha256::Hash::hash(msg);
    let msg = Message::from_slice(&msg)?;
    let id = RecoveryId::from_i32(recovery_id as i32)?;
    let sig = RecoverableSignature::from_compact(&sig, id)?;

    secp.recover(&msg, &sig)
}

fn sign_recovery<C: Signing>(secp: &Secp256k1<C>, msg: &[u8], seckey: [u8; 32]) -> Result<RecoverableSignature, Error> {
    let msg = sha256::Hash::hash(msg);
    let msg = Message::from_slice(&msg)?;
    let seckey = SecretKey::from_slice(&seckey)?;
    Ok(secp.sign_recoverable(&msg, &seckey))
}

fn main() {
    let secp = Secp256k1::new();

    let seckey = [
        59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107,
        94, 203, 174, 253, 102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28,
    ];
    let pubkey = PublicKey::from_slice(&[
        2,
        29, 21, 35, 7, 198, 183, 43, 14, 208, 65, 139, 14, 112, 205, 128, 231,
        245, 41, 91, 141, 134, 245, 114, 45, 63, 82, 19, 251, 210, 57, 79, 54,
    ]).unwrap();
    let msg = b"This is some message";

    let signature = sign_recovery(&secp, msg, seckey).unwrap();

    let (recovery_id, serialize_sig) = signature.serialize_compact();

    assert_eq!(
        recover(&secp, msg, serialize_sig, recovery_id.to_i32() as u8),
        Ok(pubkey)
    );
}
