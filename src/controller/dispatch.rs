use askama::Template;

use crate::service::grin;
use crate::{Action, Screen, State};

use crate::template::templates::SeedTemplate;

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
        },
        Action::Create(id, username) => {
            let base_dir = "/tmp/wallets";
            let message = match grin::new_wallet(&username, base_dir, "") {
                Ok(seed) => SeedTemplate { seed: &seed }.render().unwrap(),
                Err(e) => format!("Error: {}", e),
            };

            State {
                screen: Screen::Create,
                id: Some(*id),
                message: Some(message),
                ..s
            }
        }
        Action::Send(id, username, amount, destination) => {
            let message = match grin::send(
                username,
                *amount,
                destination.as_str(),
                &s.context.http_client,
            ) {
                Ok(msg) => format!("<b>Success:</b>\n{}", msg),
                Err(e) => format!("Error: {}", e),
            };

            State {
                screen: Screen::Send,
                id: Some(*id),
                message: Some(message),
                ..s
            }
        }
        Action::Help(id) => State {
            screen: Screen::Help,
            id: Some(*id),
            message: None,
            ..s
        },
        Action::NoUsername(id) => State {
            id: Some(*id),
            message: Some("You must have a username to use GrinBot.".into()),
            ..s
        },
        Action::ModeNotSupported(id) => State {
            id: Some(*id),
            message: Some(
                "For security reasons, inline and group messages are not supported.".into(),
            ),
            ..s
        },
        Action::Back(id) => State {
            prev_screen: Screen::Home,
            id: Some(*id),
            message: None,
            ..s
        },
        Action::CommandError(id, error) => State {
            id: Some(*id),
            message: Some(format!("Error: {:?}", error)),
            ..s
        },
        Action::Unknown(id) => State {
            id: Some(*id),
            message: None,
            ..s
        },
    }
}
