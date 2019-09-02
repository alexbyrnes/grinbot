mod controller;
mod service;

extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use telegram_bot::*;
use tokio_core::reactor::Core;
use redux_rs::Store;

use controller::dispatch::{State, Action, Screen, screen_reducer};
use service::telegram::TelegramService;

fn dispatch_command(store: &mut Store<State, Action>, command: &str, id: i64) {
    match command {
        "/home" => store.dispatch(Action::Home(id)),
        "/create" => store.dispatch(Action::Create(id)),
        "/send" => store.dispatch(Action::Send(id)),
        "/help" => store.dispatch(Action::Help(id)),
        "/back" => store.dispatch(Action::Back(id)),
        _ => store.dispatch(Action::Unknown(id)),
    }
}

fn get_new_ui(state: &State) -> SendMessage {
    let mut msg = SendMessage::new(
        ChatId::new(state.id.unwrap()),
        format!("Moving to {:?}", state.screen)
    );

    let inline_keyboard = reply_markup!(inline_keyboard,
        ["Create wallet" callback "/create", "Send" callback "/send"],
        ["Help" callback "/help", "Home" callback "/home"],
        ["Back" callback "/back"]
    );

    msg.reply_markup(inline_keyboard);
    msg
}

fn main() {
    let mut core = Core::new().unwrap();
    let ts = TelegramService::new(&core);

    let initial_state = State {
        id: None,
        prev_screen: Screen::Home,
        screen: Screen::Home
    };

    let mut store = Store::new(screen_reducer, initial_state);

    let future = ts.api.stream().for_each(|update| {
        match update.kind {
            // User sent a message
            UpdateKind::Message(message) => {
                let id: i64 = message.chat.id().into();
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let message_tokens: Vec<&str> = data.split(" ").collect();
                    let command = message_tokens[0];

                    dispatch_command(&mut store, command, id);
                    let msg = get_new_ui(store.state());
                    ts.api.spawn(msg);
                }
            }
            // User clicked a button
            UpdateKind::CallbackQuery(query) => {
                let id = query.message.chat.id().into();
                let message_tokens: Vec<&str> = query.data.split(" ").collect();
                let command = message_tokens[0];

                dispatch_command(&mut store, command, id);
                let msg = get_new_ui(store.state());
                ts.api.spawn(msg);
            }
            _ => {}
        }
        Ok(())
    });

    core.run(future).unwrap();
}

