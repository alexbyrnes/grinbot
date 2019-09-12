use url::Url;

use crate::service::types::GrinAmount;
use crate::types::Context;

/// Application state: which screen the user is on, previous screen, message to return.
#[derive(Default, Clone)]
pub struct State {
    pub screen: Screen,
    pub prev_screen: Screen,
    pub id: Option<i64>,
    pub message: Option<String>,
    pub context: Context,
}

/// Screens (the user's current view/state shown via Telegram message & keyboard).
#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Create,
    Send,
    Help,
}

impl Default for Screen {
    fn default() -> Screen {
        Screen::Home
    }
}

/// Actions that modify the application state.
#[derive(Debug, PartialEq)]
pub enum Action {
    Home(i64),
    Create(i64, String),
    Send(i64, String, GrinAmount, Url),
    Help(i64),
    NoUsername(i64),
    ModeNotSupported(i64),
    Back(i64),
    CommandError(i64, CommandParseError),
    Unknown(i64),
}

/// A parsed send command.
#[derive(Default, Clone)]
pub struct SendCommand {
    pub amount: f64,
    pub destination: Option<Url>,
}

impl SendCommand {
    /// Convert string tokens of user command parameters to valid Url and float.
    pub fn parse(command: Vec<&str>) -> Result<Self, CommandParseError> {
        use CommandParseError::*;
        if command.len() != 2 {
            return Err(CommandTooShortError);
        } else {
            let url = match Url::parse(command[1]) {
                Ok(url) => url,
                Err(_) => return Err(UrlParseError),
            };
            let amount = match command[0].parse::<f64>() {
                Ok(amount) => amount,
                Err(_) => return Err(AmountParseError),
            };
            Ok(SendCommand {
                amount,
                destination: Some(url),
            })
        }
    }
}

/// Errors associated with parsing commands
#[derive(Debug, PartialEq)]
pub enum CommandParseError {
    CommandTooShortError,
    UrlParseError,
    AmountParseError,
}
