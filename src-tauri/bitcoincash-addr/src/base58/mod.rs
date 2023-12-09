// https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/util/address.rs
pub mod errors;

use bitcoin_hashes::{sha256d::Hash as Sha256d, Hash};

use crate::*;
pub use errors::DecodingError;

const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

#[rustfmt::skip]
const BASE58_DIGITS: [Option<u8>; 128] = [
    None,     None,     None,     None,     None,     None,     None,     None,     // 0-7
    None,     None,     None,     None,     None,     None,     None,     None,     // 8-15
    None,     None,     None,     None,     None,     None,     None,     None,     // 16-23
    None,     None,     None,     None,     None,     None,     None,     None,     // 24-31
    None,     None,     None,     None,     None,     None,     None,     None,     // 32-39
    None,     None,     None,     None,     None,     None,     None,     None,     // 40-47
    None,     Some(0),  Some(1),  Some(2),  Some(3),  Some(4),  Some(5),  Some(6),  // 48-55
    Some(7),  Some(8),  None,     None,     None,     None,     None,     None,     // 56-63
    None,     Some(9),  Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), // 64-71
    Some(16), None,     Some(17), Some(18), Some(19), Some(20), Some(21), None,     // 72-79
    Some(22), Some(23), Some(24), Some(25), Some(26), Some(27), Some(28), Some(29), // 80-87
    Some(30), Some(31), Some(32), None,     None,     None,     None,     None,     // 88-95
    None,     Some(33), Some(34), Some(35), Some(36), Some(37), Some(38), Some(39), // 96-103
    Some(40), Some(41), Some(42), Some(43), None,     Some(44), Some(45), Some(46), // 104-111
    Some(47), Some(48), Some(49), Some(50), Some(51), Some(52), Some(53), Some(54), // 112-119
    Some(55), Some(56), Some(57), None,     None,     None,     None,     None,     // 120-127
];

fn from_base58_str(data: &str) -> Result<Vec<u8>, DecodingError> {
    // 11/15 is just over log_256(58)
    let mut scratch = vec![0u8; 1 + data.len() * 11 / 15];
    // Build in base 256
    for d58 in data.bytes() {
        // Compute "X = X * 58 + next_digit" in base 256
        if d58 as usize > BASE58_DIGITS.len() {
            return Err(DecodingError::InvalidChar(d58 as char));
        }
        let mut carry = match BASE58_DIGITS[d58 as usize] {
            Some(d58) => u32::from(d58),
            None => {
                return Err(DecodingError::InvalidChar(d58 as char));
            }
        };
        for d256 in scratch.iter_mut().rev() {
            carry += u32::from(*d256) * 58;
            *d256 = carry as u8;
            carry /= 256;
        }
        assert_eq!(carry, 0);
    }

    // Copy leading zeroes directly
    let mut ret: Vec<u8> = data
        .bytes()
        .take_while(|&x| x == BASE58_CHARS[0])
        .map(|_| 0)
        .collect();
    // Copy rest of string
    ret.extend(scratch.into_iter().skip_while(|&x| x == 0));
    Ok(ret)
}

fn to_base58_str(data: &[u8]) -> String {
    let mut ret = Vec::with_capacity(data.len());

    let mut leading_zero_count = 0;
    let mut leading_zeroes = true;
    // Build string in little endian with 0-58 in place of characters...
    for d256 in data {
        let mut carry = *d256 as usize;
        if leading_zeroes && carry == 0 {
            leading_zero_count += 1;
        } else {
            leading_zeroes = false;
        }

        for ch in ret.iter_mut() {
            let new_ch = *ch as usize * 256 + carry;
            *ch = (new_ch % 58) as u8;
            carry = new_ch / 58;
        }
        while carry > 0 {
            ret.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    // ... then reverse it and convert to chars
    for _ in 0..leading_zero_count {
        ret.push(0);
    }

    let out: String = ret
        .iter()
        .rev()
        .map(|ch| BASE58_CHARS[*ch as usize] as char)
        .collect();

    out
}

/// Codec allowing the encoding and decoding of Base58 addresses.
pub struct Base58Codec;

impl AddressCodec for Base58Codec {
    type EncodingError = ();
    type DecodingError = DecodingError;

    fn encode(
        raw: &[u8],
        hash_type: HashType,
        network: Network,
        _token_support: bool,
    ) -> Result<String, Self::EncodingError> {
        let addr_type_byte = match (hash_type, network) {
            (HashType::Key, Network::Main) => 0x00,
            (HashType::Key, Network::Test) => 0x6f,
            (HashType::Key, Network::Regtest) => 0x6f,
            (HashType::Script, Network::Main) => 0x05,
            (HashType::Script, Network::Test) => 0xc4,
            (HashType::Script, Network::Regtest) => 0xc4,
        };

        let mut body = Vec::with_capacity(raw.len() + 5);
        body.push(addr_type_byte);
        body.extend(raw);

        let checksum = Sha256d::hash(&body);
        body.extend(&checksum[0..4]);
        Ok(to_base58_str(&body))
    }

    fn decode(addr_str: &str) -> Result<Address, Self::DecodingError> {
        // Convert from base58
        let raw = from_base58_str(addr_str)?;
        let length = raw.len();
        if length != 25 {
            return Err(DecodingError::InvalidLength(length));
        }

        // Parse network and hash type
        let version_byte = raw[0];
        println!("VERSION byTE {}", version_byte);
        let (network, hash_type) = match version_byte {
            0x00 => (Network::Main, HashType::Key),
            0x05 => (Network::Main, HashType::Script),
            0x6f => (Network::Test, HashType::Key),
            0xc4 => (Network::Test, HashType::Script),
            _ => return Err(DecodingError::InvalidVersion(version_byte)),
        };

        // Verify checksum
        let payload = &raw[0..raw.len() - 4];
        let checksum_actual = &raw[raw.len() - 4..];
        let checksum_expected = &Sha256d::hash(payload)[0..4];
        if checksum_expected != checksum_actual {
            return Err(DecodingError::ChecksumFailed {
                expected: checksum_expected.to_vec(),
                actual: checksum_actual.to_vec(),
            });
        }
        let token_support = false;
        // Extract hash160 address and return
        let body = payload[1..].to_vec();
        Ok(Address {
            scheme: Scheme::Base58,
            body,
            hash_type,
            network,
            token_support,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin_hashes::hash160::Hash as Hash160;
    use hex;

    #[test]
    fn to_legacyaddr() {
        let pubkey_hex = "04005937fd439b3c19014d5f328df8c7ed514eaaf41c1980b8aeab461dffb23fbf3317e42395db24a52ce9fc947d9c22f54dc3217c8b11dfc7a09c59e0dca591d3";
        let pubkeyhash = Hash160::hash(&hex::decode(pubkey_hex).unwrap()).to_vec();
        let legacyaddr =
            Base58Codec::encode(&pubkeyhash, HashType::Key, Network::Main, false).unwrap();
        assert!(legacyaddr == "1NM2HFXin4cEQRBLjkNZAS98qLX9JKzjKn");
    }

    #[test]
    fn from_legacyaddr() {
        let legacyaddr = "1NM2HFXin4cEQRBLjkNZAS98qLX9JKzjKn";
        let result = Base58Codec::decode(legacyaddr).unwrap();
        let hash160 = result.as_body();
        assert!(hex::encode(hash160) == "ea2407829a5055466b27784cde8cf463167946bf");
    }

    #[test]
    fn from_legacyaddr_errors() {
        assert!(Base58Codec::decode("0").is_err());
        assert!(Base58Codec::decode("1000000000000000000000000000000000").is_err());
    }
}
