use std::{error::Error, fmt};

use serde_json::Value;

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct WalletExistsError;

impl Error for WalletExistsError {}

impl fmt::Display for WalletExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet exists")
    }
}

#[derive(Debug)]
pub struct CreateWalletError;

impl Error for CreateWalletError {}

impl fmt::Display for CreateWalletError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Can't create wallet")
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
	pub id: String,
	pub jsonrpc: String,
	pub method: String,
	pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MaybeReply {
    pub Ok: Value
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
	pub id: String,
	pub jsonrpc: String,
	pub result: MaybeReply,
	pub error: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
	pub code: i32,
	pub message: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Args {
    pub args: Option<Value>
}


#[derive(Debug, Copy, Clone)]
pub struct GrinAmount {
    amount: f64
}

impl GrinAmount {

    pub fn new(amount: f64) -> Self {
        GrinAmount {
            amount
        }
    }

    pub fn as_grin(&self) -> f64 {
        self.amount
    }

    pub fn as_nano_grin(&self) -> f64 {
        self.amount * 1_000_000_000_f64    
    }

}

#[derive(Debug, Copy, Clone)]
pub struct NanoGrinAmount {
    amount: f64
}

impl NanoGrinAmount {

    pub fn new(amount: f64) -> Self {
        NanoGrinAmount {
            amount
        }
    }

    pub fn as_nano_grin(&self) -> f64 {
        self.amount
    }

    pub fn as_grin(&self) -> f64 {
        self.amount / 1_000_000_000_f64    
    }

}

