use grinbot_core::controller::dispatch::{
    get_action, get_command, screen_reducer, tokenize_command,
};
use grinbot_core::controller::types::{LoggableState, Screen, State};
use grinbot_core::types::Context;

use crate::keybase::types::KeybaseMessageParseError;
use redux_rs::{Store, Subscription};
use regex::Regex;

use futures::executor::block_on;
use futures::prelude::*;
use futures::stream::StreamExt;
use keybase_bot_api::chat::{ChannelParams, Notification};

use keybase_bot_api::{ApiError, Bot, Chat};
use keybase_protocol::chat1::api;

use std::error::Error;

pub struct KeybaseService {}

impl KeybaseService {
    pub fn new() -> Self {
        KeybaseService {}
    }

    pub fn parse_update(
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
            Self::html_to_markdown(m)
        } else {
            "".to_string()
        };
        (id, message)
    }

    pub fn start(
        self,
        from_user: String,
        wallet_dir: String,
        owner_endpoint: String,
        wallet_password: String,
        log_config: String,
        cli_command: Option<&str>,
        key: String,
        to_user: String, // local bot & paper key user
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

        let mut bot = Bot::new(&to_user, &key).unwrap();

        let notifications = bot.listen().unwrap();
        let future = notifications.for_each(|notification| {
            // Unpack Keybase update (command from user).
            let (id, message_from_user, message) = Self::parse_update(notification).unwrap();
            // Get the action associated with the command.
            let action = get_action(id, &message_from_user, message, &from_user);
            // Dispatch the action.
            store.dispatch(action);
            // Use the updated state to return an updated UI (reply message).
            let (_id, message) = KeybaseService::get_keybase_ui(store.state());
            // Create channel parameters.
            let channel = ChannelParams {
                name: format!("{},{}", bot.username, from_user),
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

    /// Simple HTML to Markdown converter.
    fn html_to_markdown(html: &str) -> String {
        let reps = vec![(r"</?i>", "_"), (r"</?pre>", "```"), (r"</?b>", "*")];
        let mut markdown = html.to_string();
        reps.iter().for_each(|r| {
            let re = Regex::new(r.0).unwrap();
            markdown = re.replace_all(&markdown, r.1).into();
        });
        markdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grinbot_core::controller::dispatch::get_action;
    use grinbot_core::controller::types::Action;
    use keybase_bot_api::chat::Notification;

    #[test]
    fn raw_callback_query_update() {
        let json = r#"{
          "type": "chat",
          "source": "remote",
          "msg": {
            "id": 99,
            "conversation_id": "1234",
            "channel": {
              "name": "testchannel",
              "public": false,
              "members_type": "team",
              "topic_type": "chat",
              "topic_name": "testtopic"
            },
            "sender": {
              "uid": "123456",
              "username": "user123",
              "device_id": "1234567",
              "device_name": "MY_DEVICE"
            },
            "sent_at": 1569300000,
            "sent_at_ms": 1569300000000,
            "content": {
              "type": "text",
              "text": {
                "body": "/home",
                "teamMentions": null
              }
            },
            "prev": null,
            "unread": false
          },
          "pagination": {
            "next": "1",
            "previous": "0",
            "num": 1,
            "last": false
          }
        }
        "#;

        let notification = serde_json::from_str::<Notification>(json).unwrap();
        let (id, from_user, message) = KeybaseService::parse_update(Ok(notification)).unwrap();
        assert_eq!(
            Action::Home(99),
            get_action(id, &from_user, message, &"user123".to_string())
        );
    }

    #[test]
    fn raw_message_update() {
        let json = r#"{
          "type": "chat",
          "source": "remote",
          "msg": {
            "id": 99,
            "conversation_id": "1234",
            "channel": {
              "name": "testchannel",
              "public": false,
              "members_type": "team",
              "topic_type": "chat",
              "topic_name": "testtopic"
            },
            "sender": {
              "uid": "123456",
              "username": "user123",
              "device_id": "1234567",
              "device_name": "MY_DEVICE"
            },
            "sent_at": 1569300000,
            "sent_at_ms": 1569300000000,
            "content": {
              "type": "text",
              "text": {
                "body": "/back",
                "teamMentions": null
              }
            },
            "prev": null,
            "unread": false
          },
          "pagination": {
            "next": "1",
            "previous": "0",
            "num": 1,
            "last": false
          }
        }
        "#;
        let notification = serde_json::from_str::<Notification>(json).unwrap();
        let (id, from_user, message) = KeybaseService::parse_update(Ok(notification)).unwrap();
        assert_eq!(
            Action::Back(99),
            get_action(id, &from_user, message, &"user123".to_string())
        );
    }
}
