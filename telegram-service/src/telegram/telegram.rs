use futures::stream::Stream;
use grinbot_core::controller::dispatch::{
    get_action, get_command, get_new_ui, screen_reducer, tokenize_command,
};
use grinbot_core::controller::types::{LoggableState, Screen, State};
use grinbot_core::types::Context;
use redux_rs::{Store, Subscription};
use telegram_bot::*;
use tokio_core::reactor::Core;

pub struct TelegramService {}

impl TelegramService {
    pub fn new() -> Self {
        TelegramService {}
    }

    /// Returns the Action for each Telegram Update (message, callback, inline query)
    pub fn parse_update(update: Update) -> (i64, Option<String>, Option<String>) {
        match update.kind {
            // User sent a message
            UpdateKind::Message(telegram_message) => {
                let id = telegram_message.chat.id().into();
                if let MessageKind::Text { data, .. } = telegram_message.kind {
                    if let MessageChat::Private(from_user) = telegram_message.chat {
                        return (id, from_user.username, Some(data));
                    }
                }
                (id, None, None)
            }
            // User clicked a button
            UpdateKind::CallbackQuery(query) => {
                let id = query.message.chat.id().into();
                if let MessageChat::Private(from_user) = query.message.chat {
                    return (id, from_user.username, Some(query.data));
                }
                (id, None, None)
            }
            // User sent an inline (@grinbot123 send...) query
            UpdateKind::InlineQuery(query) => {
                let id = query.from.id.into();
                (id, query.from.username, Some("/unsupported".to_string()))
            }

            _ => (-1, None, None),
        }
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

        let mut core = Core::new().unwrap();
        let api = Api::configure(key).build(core.handle()).unwrap();

        let future = api.stream().for_each(|update| {
            let (id, from_user, message) = Self::parse_update(update);
            let action = get_action(id, &from_user, message, &config_user);
            store.dispatch(action);
            let reply = get_new_ui(store.state());
            api.spawn(reply);
            Ok(())
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
            core.run(future).unwrap();
            info!("Running...");
        }
    }
}

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
