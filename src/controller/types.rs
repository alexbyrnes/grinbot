use log::Level;
use url::Url;

use crate::service::types::GrinAmount;
use crate::types::Context;
use std::fmt;

/// Application state: which screen the user is on, previous screen, message to return.
#[derive(Default, Clone, Debug)]
pub struct State {
    pub screen: Screen,
    pub prev_screen: Screen,
    pub id: Option<i64>,
    pub message: Option<String>,
    pub context: Context,
    pub error_level: Option<Level>,
}

/// State that can be logged.
#[derive(Debug, Clone)]
pub struct LoggableState {
    pub screen: Screen,
    pub prev_screen: Screen,
    pub id: Option<i64>,
    pub message: Option<String>,
}

impl LoggableState {
    pub fn new(state: State) -> Self {
        LoggableState {
            screen: state.screen,
            prev_screen: state.prev_screen,
            id: state.id,
            message: state.message,
        }
    }
}

/// Screens (the user's current view/state shown via Telegram message & keyboard).
#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Create,
    Send,
    Balance,
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
    Create(i64),
    Send(i64, GrinAmount, Url),
    Balance(i64),
    Help(i64),
    NoUsername(i64),
    WrongUsername(i64),
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
    pub fn usage() -> String {
        "Wrong number of arguments.\n\nUsage: <pre>/send 0.001 http://some-recipient123.org</pre>"
            .to_string()
    }

    /// Convert string tokens of user command parameters to valid Url and float.
    pub fn parse(command: Vec<&str>) -> Result<Self, CommandParseError> {
        use CommandParseError::*;
        if command.len() != 2 {
            return Err(WrongNumberOfArgsError(SendCommand::usage()));
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
    WrongNumberOfArgsError(String),
    UrlParseError,
    AmountParseError,
}

impl fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandParseError::WrongNumberOfArgsError(msg) => write!(f, "{}", msg),
            error => write!(f, "{:?}", error),
        }
    }
}
