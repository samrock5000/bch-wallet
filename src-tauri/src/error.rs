use bitcoinsuite_core::error::DataError;
// use serde_json::error::Category;
use thiserror::Error;

use crate::coins::selection;
/// Common Wallet Errors
#[derive(Debug, Error, PartialEq)]
pub enum WalletError {
    #[error("CoinSelectionError")]
    CoinSelectionError { reason: String },
    #[error("Insufficient Value")]
    InputValueInsufficient {
        reason: String,
        amount_request: u64,
        actual: u64,
    },
    #[error("Amount: {amount} below dust minimum:{dust}")]
    DustValue { amount: u64, dust: u64 },
    #[error("utxo lookup failed {reason}")]
    DataBaseError { reason: String },
    #[error("input size exceeds tx input limit")]
    MaxInputSizeLimit,
    #[error("address decoder failed")]
    AddresssDecodeError { reason: String },
    #[error("something went wrong when calling electrum")]
    NetworkError { reason: String },
    #[error("Only BCH testnet and mainnet cointype are supported")]
    CoinType { reason: String },
    #[error("{reason}")]
    Generic { reason: String },
}

impl From<Box<dyn std::error::Error>> for WalletError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        WalletError::Generic {
            reason: value.to_string(),
        }
    }
}
impl From<DataError> for WalletError {
    fn from(value: DataError) -> Self {
        WalletError::Generic {
            reason: value.to_string(),
        }
    }
}
impl From<selection::Error> for WalletError {
    fn from(value: selection::Error) -> Self {
        WalletError::Generic {
            reason: format!("{:?}", value),
        }
    }
}

impl From<sled::Error> for WalletError {
    fn from(value: sled::Error) -> Self {
        WalletError::Generic {
            reason: value.to_string(),
        }
    }
}
impl From<serde_json::Error> for WalletError {
    fn from(value: serde_json::Error) -> Self {
        match value.classify() {
            _ => WalletError::DataBaseError {
                reason: "failure to read or write bytes on an I/O stream".to_string(),
            },
        }
    }
}
impl From<WalletError> for serde_json::Error {
    fn from(value: WalletError) -> Self {
        match value {
            _ => serde_json::Error::into(value.into()),
        }
    }
}
impl From<WalletError> for serde_json::Value {
    fn from(value: WalletError) -> Self {
        match value {
            _ => value.into(),
        }
    }
}
impl Into<WalletError> for serde_json::Value {
    fn into(self) -> WalletError {
        WalletError::Generic {
            reason: match self.as_str() {
                Some(err) => err.to_string(),
                None => "serde_json::Value to wallet Err".to_string(),
            },
        }
    }
}
