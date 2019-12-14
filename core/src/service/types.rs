use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use grin_wallet_libwallet::WalletInfo;

/// Wallet directory already exists.
#[derive(Debug)]
pub struct WalletExistsError;

impl Error for WalletExistsError {}

impl fmt::Display for WalletExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet exists")
    }
}

/// Api secret (.api_secret file) does not exist.
#[derive(Debug)]
pub struct ApiSecretMissingError;

impl Error for ApiSecretMissingError {}

impl fmt::Display for ApiSecretMissingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ".api_secret file does not exist in wallet directory")
    }
}

/// Error creating wallet.
#[derive(Debug)]
pub struct CreateWalletError;

impl Error for CreateWalletError {}

impl fmt::Display for CreateWalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't create wallet")
    }
}

/// RPC request to the Grin wallet owner API.
#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub id: String,
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

/// Imitates a library wrapper used by the Grin wallet.
#[derive(Serialize, Deserialize, Debug)]
pub enum MaybeReply {
    Ok(Value),
    Err(Value),
}

/// RPC response from the Grin wallet owner API.
#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
    pub id: String,
    pub jsonrpc: String,
    pub result: MaybeReply,
    pub error: Option<Value>,
}

/// RPC error response from the Grin wallet owner API.
#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Arguments sent to Grin wallet owner API.
#[derive(Debug, Serialize, Deserialize)]
pub struct Args {
    pub args: Option<Value>,
}

/// An amount of whole Grin.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GrinAmount {
    amount: f64,
}

impl GrinAmount {
    pub fn new(amount: f64) -> Self {
        GrinAmount { amount }
    }

    #[allow(dead_code)]
    pub fn as_grin(&self) -> f64 {
        self.amount
    }

    pub fn as_nano_grin(&self) -> f64 {
        self.amount * 1_000_000_000_f64
    }
}

/// Amount of Nano Grin (one billionth of a Grin).
#[derive(Debug, Copy, Clone)]
pub struct NanoGrinAmount {
    amount: f64,
}

impl NanoGrinAmount {
    pub fn new(amount: f64) -> Self {
        NanoGrinAmount { amount }
    }

    #[allow(dead_code)]
    pub fn as_nano_grin(&self) -> f64 {
        self.amount
    }

    pub fn as_grin(&self) -> f64 {
        self.amount / 1_000_000_000_f64
    }
}

// WalletInfo with whole Grin amounts
#[derive(Debug, Copy, Clone)]
pub struct WalletInfoGrin {
    pub last_confirmed_height: u64,
    pub minimum_confirmations: u64,
    pub total: f64,
    pub amount_awaiting_finalization: f64,
    pub amount_awaiting_confirmation: f64,
    pub amount_immature: f64,
    pub amount_currently_spendable: f64,
    pub amount_locked: f64,
}

impl WalletInfoGrin {
    pub fn new(wi: WalletInfo) -> Self {
        let WalletInfo {
            last_confirmed_height,
            minimum_confirmations,
            total,
            amount_awaiting_finalization,
            amount_awaiting_confirmation,
            amount_immature,
            amount_currently_spendable,
            amount_locked,
        } = wi;

        WalletInfoGrin {
            last_confirmed_height,
            minimum_confirmations,
            total: NanoGrinAmount::new(total as f64).as_grin(),
            amount_awaiting_finalization: NanoGrinAmount::new(amount_awaiting_finalization as f64)
                .as_grin(),
            amount_awaiting_confirmation: NanoGrinAmount::new(amount_awaiting_confirmation as f64)
                .as_grin(),
            amount_immature: NanoGrinAmount::new(amount_immature as f64).as_grin(),
            amount_currently_spendable: NanoGrinAmount::new(amount_currently_spendable as f64)
                .as_grin(),
            amount_locked: NanoGrinAmount::new(amount_locked as f64).as_grin(),
        }
    }
}
