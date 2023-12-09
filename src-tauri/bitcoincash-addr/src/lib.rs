//! # Bitcoin Cash Address Library
//!
//! A simple library providing an `Address` struct enabling
//! encoding/decoding of Bitcoin Cash addresses.
//!
//! ```
//! use bitcoincash_addr::{Address, Network, Scheme};
//!
//! fn main() {
//!     // Decode base58 address
//!     let legacy_addr = "1NM2HFXin4cEQRBLjkNZAS98qLX9JKzjKn";
//!     let mut addr = Address::decode(legacy_addr).unwrap();
//!
//!     // Change the base58 address to a test network cashaddr
//!     addr.network = Network::Test;
//!     addr.scheme = Scheme::CashAddr;
//!
//!     // Encode cashaddr
//!     let cashaddr_str = addr.encode().unwrap();
//!
//!     // bchtest:qr4zgpuznfg923ntyauyeh5v7333v72xhum2dsdgfh
//!     println!("{}", cashaddr_str);
//! }
//!
//! ```
//!

pub mod base58;
pub mod cashaddr;

pub use base58::Base58Codec;
pub use cashaddr::CashAddrCodec;

/// Bitcoin Networks.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Network {
    /// Main network.
    Main,
    /// Test network.
    Test,
    /// Regression test network.
    Regtest,
}

/// Address encoding scheme.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Scheme {
    /// Base58 encoding.
    Base58,
    /// CashAddr encoding.
    CashAddr,
}

/// Intepretation of the Hash160 bytes.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum HashType {
    /// Public key hash
    Key,
    /// Script hash
    Script,
}

/// Struct containing the bytes and metadata of a Bitcoin Cash address.
/// This is yeilded during decoding or consumed during encoding.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Address {
    /// Address bytes
    pub body: Vec<u8>,
    /// Encoding scheme
    pub scheme: Scheme,
    /// Hash type
    pub hash_type: HashType,
    /// Network
    pub network: Network,
    /// Token Support
    pub token_support: bool,
}

/// Creates an empty `Address` struct, with the `body` bytes the empty vector,
/// `Scheme::CashAddr`, `HashType::Key`, and `Network::Main`.
impl Default for Address {
    fn default() -> Self {
        Address {
            body: vec![],
            scheme: Scheme::CashAddr,
            hash_type: HashType::Key,
            network: Network::Main,
            token_support: false,
        }
    }
}

impl Address {
    /// Create a new address.
    pub fn new(
        body: Vec<u8>,
        scheme: Scheme,
        hash_type: HashType,
        network: Network,
        token_support: bool,
    ) -> Self {
        Address {
            body,
            scheme,
            hash_type,
            network,
            token_support,
        }
    }

    /// Borrow address bytes.
    pub fn as_body(&self) -> &[u8] {
        &self.body
    }

    /// Take address bytes.
    pub fn into_body(self) -> Vec<u8> {
        self.body
    }
    /// Take address bytes.
    pub fn token_support(self) -> bool {
        self.token_support
    }

    /// Attempt to convert the raw address bytes to a string.
    pub fn encode(&self) -> Result<String, cashaddr::EncodingError> {
        match self.scheme {
            Scheme::CashAddr => CashAddrCodec::encode(
                &self.body,
                self.hash_type.to_owned(),
                self.network.to_owned(),
                self.token_support,
            ),
            Scheme::Base58 => Ok(Base58Codec::encode(
                &self.body,
                self.hash_type.to_owned(),
                self.network.to_owned(),
                self.token_support,
            )
            .unwrap()), // Base58 encoding is infallible
        }
    }

    /// Attempt to convert an address string into bytes.
    pub fn decode(
        addr_str: &str,
    ) -> Result<Self, (cashaddr::DecodingError, base58::DecodingError)> {
        CashAddrCodec::decode(addr_str).or_else(|cash_err| {
            Base58Codec::decode(addr_str).map_err(|base58_err| (cash_err, base58_err))
        })
    }
}

/// A trait providing an interface for encoding and decoding the `Address` struct for each address scheme.
pub trait AddressCodec {
    type EncodingError;
    type DecodingError;

    /// Attempt to convert the raw address bytes to a string.
    fn encode(
        raw: &[u8],
        hash_type: HashType,
        network: Network,
        token: bool,
    ) -> Result<String, Self::EncodingError>;

    /// Attempt to convert the address string to bytes.
    fn decode(s: &str) -> Result<Address, Self::DecodingError>;
}
