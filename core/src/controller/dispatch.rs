extern crate telegram_bot;
use askama::Template;
use log::Level;

use crate::controller::types::{Action, Screen, SendCommand, State};
use crate::service::grin;
use crate::template::templates::{HelpTemplate, SeedTemplate};

use crate::service::types::GrinAmount;
use telegram_bot::*;

/// Main UI reducer: Returns a new State from an Action.
pub fn screen_reducer(state: &State, action: &Action) -> State {
    let s = state.clone();
    match action {
        Action::Home(id) => State {
            prev_screen: Screen::Home,
            screen: Screen::Home,
            id: Some(*id),
            message: None,
            context: s.context,
            error_level: None,
        },
        Action::Create(id) => {
            let (message, error_level) =
                match grin::new_wallet(&s.context.wallet_dir, &s.context.wallet_password) {
                    Ok(seed) => (SeedTemplate { seed: &seed }.render().unwrap(), None),
                    Err(e) => (format!("Error: {}", e), Some(Level::Error)),
                };

            State {
                screen: Screen::Create,
                id: Some(*id),
                message: Some(message),
                error_level,
                ..s
            }
        }
        Action::Send(id, amount, destination) => {
            let (message, error_level) = match grin::send(
                *amount,
                destination.as_str(),
                &s.context.wallet_dir,
                &s.context.owner_endpoint,
                &s.context.http_client,
            ) {
                Ok(msg) => (format!("<b>Success:</b>\n{}", msg), None),
                Err(e) => (format!("Error: {}", e), Some(Level::Info)),
            };

            State {
                screen: Screen::Send,
                id: Some(*id),
                message: Some(message),
                error_level,
                ..s
            }
        }
        Action::Balance(id) => {
            let (message, error_level) = match grin::balance(
                &s.context.wallet_dir,
                &s.context.owner_endpoint,
                &s.context.http_client,
            ) {
                Ok(msg) => (format!("<b>Success:</b>\n{}", msg), None),
                Err(e) => (format!("Error: {}", e), Some(Level::Info)),
            };

            State {
                screen: Screen::Balance,
                id: Some(*id),
                message: Some(message),
                error_level,
                ..s
            }
        }

        Action::Help(id) => {
            let message = Some(HelpTemplate {}.render().unwrap());
            State {
                screen: Screen::Help,
                id: Some(*id),
                message,
                error_level: None,
                ..s
            }
        }
        Action::NoUsername(id) => State {
            id: Some(*id),
            message: Some("You must have a username to use Grin Bot.".into()),
            error_level: Some(Level::Warn),
            ..s
        },
        Action::WrongUsername(id) => State {
            id: Some(*id),
            message: Some(
                "Your username does not match the username in the Grin Bot config.".into(),
            ),
            error_level: Some(Level::Warn),
            ..s
        },

        Action::ModeNotSupported(id) => State {
            id: Some(*id),
            message: Some(
                "For security reasons, inline and group messages are not supported.".into(),
            ),
            error_level: Some(Level::Warn),
            ..s
        },
        Action::Back(id) => State {
            prev_screen: Screen::Home,
            id: Some(*id),
            message: None,
            error_level: None,
            ..s
        },
        Action::CommandError(id, error) => State {
            id: Some(*id),
            message: Some(format!("Error: {}", error)),
            error_level: Some(Level::Error),
            ..s
        },
        Action::Unknown(id) => State {
            id: Some(*id),
            message: None,
            error_level: Some(Level::Error),
            ..s
        },
    }
}

/// Splits command into type and parameters
pub fn tokenize_command(raw_command: &str) -> (&str, Vec<&str>) {
    let message_tokens: Vec<&str> = raw_command.split(" ").collect();
    (message_tokens[0], message_tokens[1..].to_vec())
}

pub fn get_action(
    id: i64,
    from_user: Option<String>,
    message: Option<String>,
    config_user: &str,
) -> Action {
    if let Some(msg) = message {
        let (command_type, parameters) = tokenize_command(&msg);
        // Check for username before looking at command
        match get_username_action(id, from_user, config_user) {
            Some(action) => action,
            None => get_command(command_type, id, parameters),
        }
    } else {
        Action::Unknown(id)
    }
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

/// Dispatches a command entered by user.
///
/// # Example
///
/// ```
/// use grinbot_core::controller::dispatch::{get_command};
/// get_command("/send", 99, vec!["0.01", "http://recipient123.org"]);
/// ```
pub fn get_command(command_type: &str, id: i64, command: Vec<&str>) -> Action {
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
        "/unsupported" => Action::ModeNotSupported(id),
        _ => Action::Unknown(id),
    }
}

/// Get actions associated with usernames
pub fn get_username_action(id: i64, username: Option<String>, config_user: &str) -> Option<Action> {
    match username {
        None => Some(Action::NoUsername(id)),
        Some(current_username) => {
            if current_username != config_user {
                Some(Action::WrongUsername(id))
            } else {
                None
            }
        }
    }
}

/// Returns the next Telegram message from the current state.
pub fn get_new_ui(state: &State) -> SendMessage {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::dispatch::{get_action, get_command};
    use crate::controller::types::{Action, SendCommand};

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
        let command = get_username_action(101, None, &"user123".to_string());
        assert_eq!(command, Some(Action::NoUsername(101)));
    }

    #[test]
    fn wrong_username() {
        let command = get_username_action(101, Some("user321".to_string()), &"user123".to_string());
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
        use crate::controller::types::CommandParseError::*;
        let command = get_command("/send", 103, vec!["0.01"]);
        assert_eq!(
            command,
            Action::CommandError(103, WrongNumberOfArgsError(SendCommand::usage()))
        );
    }

    #[test]
    fn no_amount_send_command() {
        use crate::controller::types::CommandParseError::*;
        let command = get_command("/send", 103, vec!["https://recipient123.org"]);
        assert_eq!(
            command,
            Action::CommandError(103, WrongNumberOfArgsError(SendCommand::usage()))
        );
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
