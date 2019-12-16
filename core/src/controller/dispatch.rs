use askama::Template;
use log::Level;

use crate::controller::types::{Action, Screen, SendCommand, State};
use crate::service::grin;
use crate::template::templates::{HelpTemplate, SeedTemplate};

use crate::service::types::GrinAmount;

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
    from_user: &Option<String>,
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
pub fn get_username_action(
    id: i64,
    username: &Option<String>,
    config_user: &str,
) -> Option<Action> {
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

}
