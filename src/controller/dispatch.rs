use askama::Template;
use log::Level;

use crate::service::grin;
use crate::template::templates::{HelpTemplate, SeedTemplate};
use crate::{Action, Screen, State};

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
                Err(e) => (
                    format!(
                        "Error: {}\nUsage: <pre>/send 0.001 http://some-recipient123.org</pre>",
                        e
                    ),
                    Some(Level::Info),
                ),
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
            message: Some("You must have a username to use GrinBot.".into()),
            error_level: Some(Level::Warn),
            ..s
        },
        Action::WrongUsername(id) => State {
            id: Some(*id),
            message: Some(
                "Your username does not match the username in the GrinBot config.".into(),
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
            message: Some(format!("Error: {:?}", error)),
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
