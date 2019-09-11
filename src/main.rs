mod controller;
mod service;
mod template;

/// Application-level types.
mod types;

extern crate futures;
extern crate reqwest;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use redux_rs::Store;
use telegram_bot::*;
use tokio_core::reactor::Core;

use controller::dispatch::screen_reducer;
use controller::types::{Action, Screen, SendCommand, State};
use service::telegram::TelegramService;
use service::types::GrinAmount;
use types::Context;

/// Dispatches a command entered by user.
///
/// # Example
///
/// ```
/// get_command("/send", 99, Some("user_user123".into()), vec!["0.01", "http://recipient123.org"]);
/// ```
fn get_command(
    command_type: &str,
    id: i64,
    username: Option<String>,
    command: Vec<&str>,
) -> Action {
    if username.is_none() {
        return Action::NoUsername(id);
    }

    match command_type {
        "/home" => Action::Home(id),
        "/create" => Action::Create(id, username.unwrap()),
        "/send" => match SendCommand::parse(command) {
            Ok(send_command) => {
                let amount = GrinAmount::new(send_command.amount);
                let url = send_command.destination.unwrap();
                Action::Send(id, username.unwrap(), amount, url)
            }
            Err(error) => Action::CommandError(id, error),
        },
        "/help" => Action::Help(id),
        "/back" => Action::Back(id),
        _ => Action::Unknown(id),
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
            format!("Moving to {:?}", state.screen)
        },
    );

    let keyboard = reply_markup!(
        reply_keyboard,
        selective,
        one_time,
        resize,
        ["/create", "/send"],
        ["/help", "/home"],
        ["/back"]
    );

    msg.parse_mode(ParseMode::Html);
    msg.reply_markup(keyboard);
    msg
}

fn main() {
    let mut core = Core::new().unwrap();
    let http_client = reqwest::Client::new();

    let context = Context { http_client };
    let ts = TelegramService::new(&core);

    let initial_state = State {
        id: None,
        prev_screen: Screen::Home,
        screen: Screen::Home,
        message: None,
        context: context,
    };

    let mut store = Store::new(screen_reducer, initial_state);

    let future = ts.api.stream().for_each(|update| {
        match update.kind {
            // User sent a message
            UpdateKind::Message(message) => {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let message_tokens: Vec<&str> = data.split(" ").collect();
                    let command_type = message_tokens[0];
                    let command = message_tokens[1..].to_vec();
                    let id = message.chat.id().into();
                    if let MessageChat::Private(user) = message.chat {
                        let command = get_command(command_type, id, user.username, command);
                        store.dispatch(command);
                        let msg = get_new_ui(store.state());
                        ts.api.spawn(msg);
                    }
                }
            }
            // User clicked a button
            UpdateKind::CallbackQuery(query) => {
                let message_tokens: Vec<&str> = query.data.split(" ").collect();
                let command_type = message_tokens[0];
                let command = message_tokens[1..].to_vec();
                let id = query.message.chat.id().into();
                if let MessageChat::Private(user) = query.message.chat {
                    let command = get_command(command_type, id, user.username, command);
                    store.dispatch(command);
                    let msg = get_new_ui(store.state());
                    ts.api.spawn(msg);
                }
            }
            _ => {}
        }
        Ok(())
    });

    core.run(future).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_command() {
        let command = get_command("/home", 99, Some("user123".into()), vec![]);
        assert_eq!(command, Action::Home(99));
    }

    #[test]
    fn test_unknown_command() {
        let command = get_command("/abcd", 100, Some("user123".into()), vec![]);
        assert_eq!(command, Action::Unknown(100));
    }

    #[test]
    fn test_no_username_command() {
        let command = get_command("/send", 101, None, vec![]);
        assert_eq!(command, Action::NoUsername(101));
    }

    #[test]
    fn test_send_command() {
        use url::Url;

        let command = get_command(
            "/send",
            102,
            Some("user123".into()),
            vec!["0.01", "https://recipient123.org"],
        );
        let url = Url::parse("https://recipient123.org").ok().unwrap();
        assert_eq!(
            command,
            Action::Send(102, "user123".into(), GrinAmount::new(0.01), url)
        );
    }

    #[test]
    fn test_no_recipient_send_command() {
        use controller::types::CommandParseError::*;
        let command = get_command("/send", 103, Some("user123".into()), vec!["0.01"]);
        assert_eq!(command, Action::CommandError(103, CommandTooShortError));
    }

    #[test]
    fn test_no_amount_send_command() {
        use controller::types::CommandParseError::*;
        let command = get_command(
            "/send",
            103,
            Some("user123".into()),
            vec!["https://recipient123.org"],
        );
        assert_eq!(command, Action::CommandError(103, CommandTooShortError));
    }

}
