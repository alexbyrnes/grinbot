mod controller;
mod service;
mod template;

/// Application-level types.
mod types;

extern crate clap;
extern crate futures;
extern crate reqwest;
extern crate telegram_bot;
extern crate tokio_core;
extern crate yaml_rust;

#[macro_use]
extern crate log;
extern crate log4rs;

use clap::{App, Arg};
use futures::Stream;
use redux_rs::{Store, Subscription};
use std::fs::File;
use std::io::prelude::*;
use telegram_bot::*;
use tokio_core::reactor::Core;
use yaml_rust::{Yaml, YamlLoader};

use controller::dispatch::screen_reducer;
use controller::types::{Action, LoggableState, Screen, SendCommand, State};
use service::telegram::TelegramService;
use service::types::GrinAmount;
use types::Context;

/// Dispatches a command entered by user.
///
/// # Example
///
/// ```
/// get_command("/send", 99, vec!["0.01", "http://recipient123.org"]);
/// ```
fn get_command(command_type: &str, id: i64, command: Vec<&str>) -> Action {
    match command_type {
        "/home" => Action::Home(id),
        "/create" => Action::Create(id),
        "/send" => match SendCommand::parse(command) {
            Ok(send_command) => {
                let amount = GrinAmount::new(send_command.amount);
                let url = send_command.destination.unwrap();
                Action::Send(id, amount, url)
            }
            Err(error) => Action::CommandError(id, error),
        },
        "/balance" => Action::Balance(id),
        "/help" => Action::Help(id),
        "/start" => Action::Help(id),
        "/back" => Action::Back(id),
        _ => Action::Unknown(id),
    }
}

/// Get actions associated with usernames
fn get_username_action(
    id: i64,
    username: &Option<String>,
    config_username: &String,
) -> Option<Action> {
    match username {
        None => Some(Action::NoUsername(id)),
        Some(current_username) => {
            if current_username != config_username {
                Some(Action::WrongUsername(id))
            } else {
                None
            }
        }
    }
}

/// Returns the next Telegram message from the current state.
///
/// # Example
///
/// ```
/// let msg = get_new_ui(store.state());
/// ```
fn get_new_ui(state: &State) -> SendMessage {
    let mut msg = SendMessage::new(
        ChatId::new(state.id.unwrap()),
        if let Some(m) = &state.message {
            format!("{}", m)
        } else {
            "".to_string()
        },
    );
    let keyboard = reply_markup!(
        reply_keyboard,
        selective,
        one_time,
        resize,
        ["/balance", "/help"]
    );

    msg.parse_mode(ParseMode::Html);
    msg.reply_markup(keyboard);
    msg
}

fn load_config_field(config: &Yaml, field: &str) -> String {
    config[field]
        .as_str()
        .expect(&format!("{} required in config.yml", field))
        .to_string()
}

fn main() {
    // Parse optional chat message from command line
    let matches = App::new("GrinBot")
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

    // Get logging config
    let log_config = load_config_field(config, "log_config");

    // Get bot key
    let key = load_config_field(config, "telegram_bot_key");

    // Get wallet directory, either current or
    // future after create command.
    let wallet_dir = load_config_field(config, "wallet_dir");

    // Get owner API endpoint
    let owner_endpoint = load_config_field(config, "owner_endpoint");

    // Get wallet password. The password used
    // when creating a wallet and with grin-wallet commands.
    let wallet_password = load_config_field(config, "wallet_password");

    // Get username. This is the only user who may use
    // the wallet.
    let config_username = load_config_field(config, "username");

    // Logging
    log4rs::init_file(log_config, Default::default()).unwrap();
    info!("Starting GrinBot...");
    let logging_listener: Subscription<State> = |state: &State| {
        // Log actions with a log level
        if let Some(level) = state.error_level {
            log!(level, "{:#?}", LoggableState::new(state.clone()));
        }
    };

    // Initialize tokio and telegram service
    let mut core = Core::new().unwrap();
    let ts = TelegramService::new(&core, key.into());

    // Initialize reqwest and app context
    let http_client = reqwest::Client::new();
    let context = Context {
        http_client,
        wallet_dir,
        owner_endpoint,
        wallet_password,
    };

    // Initial state of the bot
    let initial_state = State {
        id: None,
        prev_screen: Screen::Home,
        screen: Screen::Home,
        message: None,
        context: context,
        error_level: None,
    };

    // The state management store
    let mut store = Store::new(screen_reducer, initial_state);

    // Log actions
    store.subscribe(logging_listener);

    // Main app loop. Ingest telegram Updates (chats),
    // dispatch associated action, get reply interface
    // with message and keyboard, and reply.
    let future = ts.api.stream().for_each(|update| {
        let action = get_action(update, &config_username);
        store.dispatch(action);
        let msg = get_new_ui(store.state());
        ts.api.spawn(msg);
        Ok(())
    });

    // Run the command line update, if any.
    if let Some(command) = cli_command {
        let (command_type, parameters) = tokenize_command(command);
        let action = get_command(command_type, 0, parameters);
        store.dispatch(action);
        let message = &store.state().message;
        println!("{}", message.clone().unwrap());
    } else {
        // Start main loop
        core.run(future).unwrap();
        info!("Running...");
    }
}

/// Splits command into type and parameters
fn tokenize_command(raw_command: &str) -> (&str, Vec<&str>) {
    let message_tokens: Vec<&str> = raw_command.split(" ").collect();
    (message_tokens[0], message_tokens[1..].to_vec())
}

/// Returns the Action for each Telegram Update (message, callback, inline query)
fn get_action(update: Update, config_username: &String) -> Action {
    match update.kind {
        // User sent a message
        UpdateKind::Message(message) => {
            let id = message.chat.id().into();
            if let MessageKind::Text { ref data, .. } = message.kind {
                let (command_type, parameters) = tokenize_command(data);

                if let MessageChat::Private(user) = message.chat {
                    // Check for username before looking at command
                    match get_username_action(id, &user.username, config_username) {
                        Some(action) => return action,
                        None => return get_command(command_type, id, parameters),
                    };
                }
            }

            Action::Unknown(id)
        }
        // User clicked a button
        UpdateKind::CallbackQuery(query) => {
            let id = query.message.chat.id().into();
            let (command_type, parameters) = tokenize_command(&query.data);
            if let MessageChat::Private(user) = query.message.chat {
                // Check for username before looking at command
                match get_username_action(id, &user.username, &config_username) {
                    Some(action) => return action,
                    None => return get_command(command_type, id, parameters),
                };
            }
            Action::Unknown(id)
        }
        // User sent an inline (@grinbot123 send...) query
        UpdateKind::InlineQuery(query) => {
            let id = query.from.id.into();
            Action::ModeNotSupported(id)
        }

        _ => Action::Unknown(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_command() {
        let command = get_command("/home", 99, vec![]);
        assert_eq!(command, Action::Home(99));
    }

    #[test]
    fn balance_command() {
        let command = get_command("/balance", 99, vec![]);
        assert_eq!(command, Action::Balance(99));
    }

    #[test]
    fn unknown_command() {
        let command = get_command("/abcd", 100, vec![]);
        assert_eq!(command, Action::Unknown(100));
    }

    #[test]
    fn no_username() {
        let command = get_username_action(101, &None, &"user123".to_string());
        assert_eq!(command, Some(Action::NoUsername(101)));
    }

    #[test]
    fn wrong_username() {
        let command =
            get_username_action(101, &Some("user321".to_string()), &"user123".to_string());
        assert_eq!(command, Some(Action::WrongUsername(101)));
    }

    #[test]
    fn send_command() {
        use url::Url;

        let command = get_command("/send", 102, vec!["0.01", "https://recipient123.org"]);
        let url = Url::parse("https://recipient123.org").ok().unwrap();
        assert_eq!(command, Action::Send(102, GrinAmount::new(0.01), url));
    }

    #[test]
    fn no_recipient_send_command() {
        use controller::types::CommandParseError::*;
        let command = get_command("/send", 103, vec!["0.01"]);
        assert_eq!(command, Action::CommandError(103, CommandTooShortError));
    }

    #[test]
    fn no_amount_send_command() {
        use controller::types::CommandParseError::*;
        let command = get_command("/send", 103, vec!["https://recipient123.org"]);
        assert_eq!(command, Action::CommandError(103, CommandTooShortError));
    }

    #[test]
    fn raw_inline_query_update() {
        let json = r#"{
                  "update_id": 999999,
                  "inline_query": {
                    "id": "9999",
                    "from": {
                       "id":99,
                       "username":"user123",
                       "first_name":"firstname",
                       "last_name":"lastname",
                       "type": "private",
                       "is_bot": false,
                       "language_code":"en"
                    },
                    "query": "/send",
                    "offset": ""
                  }
                }
            "#;
        let update = serde_json::from_str::<Update>(json).unwrap();
        assert_eq!(
            Action::ModeNotSupported(99),
            get_action(update, &"user123".to_string())
        );
    }

    #[test]
    fn raw_callback_query_update() {
        let json = r#"{
                "update_id": 999999,
                "message": {
                  "message_id": 9999,
                  "from": {
                    "id": 99,
                    "is_bot": false,
                    "first_name": "firstname",
                    "username": "user123",
                    "language_code": "en"
                  },
                  "chat": {
                    "id": 99,
                    "first_name": "firstname",
                    "username": "user123",
                    "type": "private"
                  },
                  "date": 1568300000,
                  "text": "/home",
                  "entities": [
                    {
                      "offset": 0,
                      "length": 5,
                      "type": "bot_command"
                    }]
                }
            }"#;
        let update = serde_json::from_str::<Update>(json).unwrap();
        assert_eq!(Action::Home(99), get_action(update, &"user123".into()));
    }

    #[test]
    fn raw_message_update() {
        let json = r#"{
            "update_id":999999,
              "message":{
                "date": 1568300000,
                "chat":{
                   "id":99,
                   "username":"user123",
                   "first_name":"firstname",
                   "last_name":"lastname",
                   "type": "private"
                },
                "message_id":9999,
                "from":{
                   "id":99,
                   "username":"firstlast",
                   "first_name":"firstname",
                   "last_name":"lastname",
                   "type": "private",
                   "is_bot": false
                },
                "text":"/back"
              }
            }"#;
        let update = serde_json::from_str::<Update>(json).unwrap();
        assert_eq!(Action::Back(99), get_action(update, &"user123".into()));
    }
}
