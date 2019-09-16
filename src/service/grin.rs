use askama::Template;
use reqwest::Client;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Child, Command};

use grin_wallet_libwallet::{InitTxArgs, InitTxSendArgs};

use crate::service::types::{
    ApiSecretMissingError, Args, CreateWalletError, GrinAmount, MaybeReply, NanoGrinAmount,
    RpcRequest, RpcResponse, WalletExistsError,
};
use crate::template::templates::SendSuccessTemplate;

/// Sends Grin
pub fn send(
    amount: GrinAmount,
    dest: &str,
    wallet_dir: &str,
    client: &Client,
) -> Result<String, Box<dyn Error>> {
    let owner_endpoint = "http://127.0.0.1:3420/v2/owner";

    let ita = InitTxArgs {
        src_acct_name: Some("default".into()),
        amount: amount.as_nano_grin() as u64, // Conversion occurs here
        minimum_confirmations: 10,
        max_outputs: 500,
        num_change_outputs: 1,
        selection_strategy_is_use_all: false,
        message: None,
        target_slate_version: None,
        estimate_only: None,
        send_args: Some(InitTxSendArgs {
            method: "http".into(),
            dest: dest.to_string(),
            finalize: true,
            post_tx: true,
            fluff: false,
        }),
    };

    let args = Args {
        args: Some(serde_json::to_value(&ita).unwrap()),
    };

    let rpc_request = RpcRequest {
        id: "1".to_owned(),
        jsonrpc: "2.0".to_owned(),
        method: "init_send_tx".to_owned(),
        params: Some(serde_json::to_value(&args).unwrap()),
    };

    let api_secret = get_api_secret(wallet_dir)?;
    let mut raw_response = client
        .post(owner_endpoint)
        .basic_auth("grin", Some(api_secret))
        .json(&rpc_request)
        .send()?;

    let response: RpcResponse = raw_response.json()?;
    match response.result {
        MaybeReply::Ok(rpc) => {
            let amount =
                NanoGrinAmount::new(rpc["amount"].as_str().unwrap().parse::<f64>()?).as_grin();

            let fee = NanoGrinAmount::new(rpc["fee"].as_str().unwrap().parse::<f64>()?).as_grin();

            let block_height = rpc["height"].as_str().unwrap();
            let id = rpc["id"].as_str().unwrap();
            let message = SendSuccessTemplate {
                amount,
                fee,
                block_height,
                id,
            }
            .render()
            .unwrap();
            Ok(message)
        }
        MaybeReply::Err(e) => Ok(serde_json::to_string_pretty(&e)?),
    }
}

/// Generates new wallet and returns the seed.
pub fn new_wallet(wallet_dir: &str, password: &str) -> Result<String, Box<dyn Error>> {
    let your_recovery_phrase = "Your recovery phrase is:";

    let path = Path::new(&wallet_dir);
    if !Path::exists(path) {
        Command::new("mkdir").arg(wallet_dir).output()?;

        let output = Command::new("grin-wallet")
            .current_dir(wallet_dir)
            .args(&["-p", password, "init", "-h"])
            .output()?;

        let utf8_output = String::from_utf8_lossy(&output.stdout).to_string();
        let lines: Vec<&str> = utf8_output.split("\n").collect();
        let seed_index = lines.iter().position(|&r| r == your_recovery_phrase);
        if let Some(i) = seed_index {
            Ok(lines[i + 2].to_string())
        } else {
            Err(Box::new(CreateWalletError))
        }
    } else {
        Err(Box::new(WalletExistsError))
    }
}

/// Gets .api_secret contents from wallet directory
/// for logging into owner api.
pub fn get_api_secret(wallet_dir: &str) -> Result<String, Box<dyn Error>> {
    let secret_path = wallet_dir.to_string() + "/.api_secret";

    if let Ok(mut file) = File::open(&secret_path) {
        let mut secret = String::new();
        file.read_to_string(&mut secret)?;
        Ok(secret)
    } else {
        Err(Box::new(ApiSecretMissingError))
    }
}

pub fn start_owner_api(wallet_dir: &str, password: &str) -> Result<Child, std::io::Error> {
    Command::new("grin-wallet")
        .current_dir(wallet_dir)
        .args(&["-p", password, "owner_api"])
        .spawn()
}
