use clap::{App, Arg};
use yaml_rust::{Yaml, YamlLoader};

use std::fs::File;
use std::io::prelude::*;

use keybase_service::keybase::KeybaseService;
use telegram_service::telegram::TelegramService;

fn load_config_field(config: &Yaml, field: &str) -> String {
    config[field]
        .as_str()
        .expect(&format!("{} required in config.yml", field))
        .to_string()
}

fn main() {
    // Parse optional chat message from command line
    let matches = App::new("Grin Bot")
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("commmand")
                .value_name("COMMAND")
                .help("Runs a chat command locally without Telegram")
                .takes_value(true),
        )
        .get_matches();
    let cli_command = matches.value_of("command");

    // Load config file
    let mut f = File::open(&"config.yml").expect("config.yml must exist in current directory.");
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let yml = YamlLoader::load_from_str(&s).unwrap();
    let config = &yml[0];

    // Get mode: Keybase (default) or Telegram
    let telegram_mode: bool = config["telegram_mode"]
        .as_bool()
        .expect(&format!("telegram_mode required in config.yml"));

    // Get logging config
    let log_config = load_config_field(config, "log_config");

    // Get wallet directory, either current or
    // future after create command.
    let wallet_dir = load_config_field(config, "wallet_dir");

    // Get owner API endpoint
    let owner_endpoint = load_config_field(config, "owner_endpoint");

    // Get wallet password. The password used
    // when creating a wallet and with grin-wallet commands.
    let wallet_password = load_config_field(config, "wallet_password");

    if telegram_mode {
        // Get telegram username.
        // This is the only user who may use the wallet.
        let telegram_from_user = load_config_field(config, "telegram_from_user");

        // Get bot key
        let telegram_bot_key = load_config_field(config, "telegram_bot_key");

        // Initialize and start telegram service
        let ts: TelegramService = TelegramService::new();
        ts.start(
            telegram_from_user,
            wallet_dir.clone(),
            owner_endpoint.clone(),
            wallet_password.clone(),
            log_config.clone(),
            cli_command,
            telegram_bot_key,
        );
    } else {
        // Get keybase username.
        // This is the only user who may use the wallet.
        let keybase_from_user = load_config_field(config, "keybase_from_user");

        // Get keybase paper key
        let keybase_paper_key = load_config_field(config, "keybase_paper_key");

        // Get local keybase bot user.
        // This is the user associated with the paper key.
        // May be the same as the "from" (usually mobile) user.
        let keybase_to_user = load_config_field(config, "keybase_to_user");

        // Initialize and start keybase service
        let ks: KeybaseService = KeybaseService::new();
        ks.start(
            keybase_from_user,
            wallet_dir,
            owner_endpoint,
            wallet_password,
            log_config,
            cli_command,
            keybase_paper_key,
            keybase_to_user,
        );
    }
}
