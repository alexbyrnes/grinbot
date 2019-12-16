use grinbot_core::controller::dispatch::{
    get_action, get_command, screen_reducer, tokenize_command,
};
use grinbot_core::controller::types::{LoggableState, Screen, State};
use grinbot_core::types::Context;

use crate::keybase::types::KeybaseMessageParseError;
use redux_rs::{Store, Subscription};

use futures::executor::block_on;
use futures::prelude::*;
use futures::stream::StreamExt;
use keybase_bot_api::chat::{ChannelParams, Notification};

use keybase_bot_api::{ApiError, Bot, Chat};
use keybase_protocol::chat1::{api, MsgSummary};

use std::error::Error;

pub struct KeybaseService {}

impl KeybaseService {
    pub fn new() -> Self {
        KeybaseService {}
    }

    fn parse_update(
        notification: Result<Notification, ApiError>,
    ) -> Result<(i64, Option<String>, Option<String>), Box<dyn Error>> {
        match notification.unwrap() {
            Notification::Chat(api::MsgNotification { msg, .. }) => {
                let summary = msg.ok_or(KeybaseMessageParseError)?;
                let id = summary.id.ok_or(KeybaseMessageParseError)?;
                let from_user = summary.sender.ok_or(KeybaseMessageParseError)?.username;
                let message = summary
                    .content
                    .ok_or(KeybaseMessageParseError)?
                    .text
                    .ok_or(KeybaseMessageParseError)?
                    .body;
                Ok((id as i64, from_user, message))
            }
            _ => Err(Box::new(KeybaseMessageParseError)),
        }
    }

    /// Returns the next message from the current state.
    pub fn get_keybase_ui(state: &State) -> (i64, String) {
        let id = state.id.unwrap();
        let message = if let Some(m) = &state.message {
            format!("{}", m)
        } else {
            "".to_string()
        };
        (id, message)
    }

    pub fn start(
        self,
        config_user: String,
        wallet_dir: String,
        owner_endpoint: String,
        wallet_password: String,
        log_config: String,
        cli_command: Option<&str>,
        key: String,
    ) {
        // Logging
        log4rs::init_file(log_config, Default::default()).unwrap();
        info!("Starting Grin Bot...");
        let logging_listener: Subscription<State> = |state: &State| {
            // Log actions with a log level
            if let Some(level) = state.error_level {
                log!(level, "{:#?}", LoggableState::new(state.clone()));
            }
        };

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

        let mut bot = Bot::new(&config_user, &key).unwrap();

        let notifications = bot.listen().unwrap();
        let future = notifications.for_each(|notification| {
            // Unpack Keybase update (command from user).
            let (id, from_user, message) = Self::parse_update(notification).unwrap();
            // Get the action associated with the command.
            let action = get_action(id, &from_user, message, &config_user);
            // Dispatch the action.
            store.dispatch(action);
            // Use the updated state to return an updated UI (reply message).
            let (id, message) = KeybaseService::get_keybase_ui(store.state());
            // Create channel parameters.
            let channel = ChannelParams {
                name: format!("{},{}", bot.username, from_user.unwrap()),
                ..Default::default()
            };
            // Send reply to user.
            if let Err(e) = bot.send_msg(&channel, &message) {
                println!("Failed to send message: {:?}", e);
            }
            future::ready(())
        });

        // Run the command line update, if any.
        if let Some(command) = cli_command {
            let (command_type, parameters) = tokenize_command(&command);
            let action = get_command(command_type, 0, parameters);
            store.dispatch(action);
            let message = &store.state().message;
            println!("{}", message.clone().unwrap());
        } else {
            // Start main loop
            block_on(future);
            info!("Running...");
        }
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use grinbot_core::controller::dispatch::get_action;
    use grinbot_core::controller::types::Action;
    use telegram_bot::Update;

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
        let (id, from_user, message) = parse_update(update);
        assert_eq!(
            Action::ModeNotSupported(99),
            get_action(id, from_user, message, &"user123".to_string())
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
        let (id, from_user, message) = parse_update(update);
        assert_eq!(
            Action::Home(99),
            get_action(id, from_user, message, &"user123".to_string())
        );
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
        let (id, from_user, message) = parse_update(update);
        assert_eq!(
            Action::Back(99),
            get_action(id, from_user, message, &"user123".to_string())
        );
    }
}
*/
