use url::Url;

use std::{error::Error, fmt};

use crate::service::types::GrinAmount;
use crate::types::Context;

#[derive(Default, Clone)]
pub struct State {
    pub screen: Screen, 
    pub prev_screen: Screen,
    pub id: Option<i64>,
    pub message: Option<String>,
    pub context: Context 
}

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Create,
    Send,
    Help
}

impl Default for Screen {
    fn default() -> Screen {
        Screen::Home
    }
}

#[derive(Debug)]
pub enum Action {
    Home(i64),
    Create(i64, String),
    Send(i64, String, GrinAmount, Url),
    Help(i64),
    NoUsername(i64),
    Back(i64),
    CommandError(i64, Box<dyn Error>),
    Unknown(i64)
}


#[derive(Debug)]
struct CommandTooShortError;

impl Error for CommandTooShortError {}

impl fmt::Display for CommandTooShortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command too short")
    }
}



#[derive(Default, Clone)]
pub struct SendCommand {
    pub amount: f64,
    pub destination: Option<Url>,
}

impl SendCommand {

    pub fn parse(command: Vec<&str>) -> Result<Self, Box<dyn Error>> {
        if command.len() != 2 {
            return Err(Box::new(CommandTooShortError));
        } else {
            let url = Url::parse(command[1])?;
            Ok(SendCommand {
                amount: command[0].parse::<f64>()?, 
                destination: Some(url) 
            })

        }

    }
}

