mod controller;
mod service;
mod template;
mod types;

extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;
extern crate reqwest;

use futures::Stream;
use telegram_bot::*;
use tokio_core::reactor::Core;
use redux_rs::Store;
use url::Url;

use controller::types::{State, Action, Screen, SendCommand};
use types::Context;
use service::telegram::TelegramService;
use service::types::GrinAmount;
use controller::dispatch::screen_reducer;



fn dispatch_command(store: &mut Store<State, Action>, command_type: &str, id: i64, username: Option<String>, command: Vec<&str>) {
    if username.is_none() {
        store.dispatch(Action::NoUsername(id));
        return;
    }

    match command_type {
        "/home" => store.dispatch(Action::Home(id)),
        "/create" => store.dispatch(Action::Create(id, username.unwrap())),
        "/send" => {
            match SendCommand::parse(command) {
                Ok(send_command) => {
                    let amount = GrinAmount::new(send_command.amount);
                    let url = send_command.destination.unwrap();
                    store.dispatch(Action::Send(id, username.unwrap(), amount, url));
                }
                Err(error) => store.dispatch(Action::CommandError(id, error))
            }
        }
        "/help" => store.dispatch(Action::Help(id)),
        "/back" => store.dispatch(Action::Back(id)),
        _ => store.dispatch(Action::Unknown(id)),
    }
}

fn get_new_ui(state: &State) -> SendMessage {
    let mut msg = SendMessage::new(
        ChatId::new(state.id.unwrap()),
        if let Some(m) = &state.message {
            format!("{}", m)
        } else {
            format!("Moving to {:?}", state.screen)
        },
    );

    let keyboard = reply_markup!(reply_keyboard, selective, one_time, resize,
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
        context: context
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
                        dispatch_command(&mut store, command_type, id, user.username, command);
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
                    dispatch_command(&mut store, command_type, id, user.username, command);
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

