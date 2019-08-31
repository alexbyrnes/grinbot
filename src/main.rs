extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use futures::Stream;
use telegram_bot::*;
use tokio_core::reactor::Core;

use std::env;

fn main() {
    let mut core = Core::new().unwrap();

    let token = env::var("TELEGRAM_BOT_KEY").expect("TELEGRAM_BOT_KEY not set");
    let api = Api::configure(token).build(core.handle()).unwrap();

    let future = api.stream().for_each(|update| {
        match update.kind {
            // User sent a message
            UpdateKind::Message(message) => {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    let mut msg = SendMessage::new(
                        message.chat, 
                        format!("You sent {:?} \nWhat would you like to do?", data)
                    );

                    let inline_keyboard = reply_markup!(inline_keyboard,
                        ["Create wallet" callback "create_wallet", "Send" callback "send"],
                        ["Help" callback "help"]
                    );

                    msg.reply_markup(inline_keyboard);
                    api.spawn(msg);
                }
            }
            // User clicked a button
            UpdateKind::CallbackQuery(query) => {
                let msg = SendMessage::new(
                    query.message.chat, 
                    format!("command {}, id {:?}", query.data, query.id)
                );

                api.spawn(msg);
            }
            _ => {}
        }
        Ok(())
    });

    core.run(future).unwrap();
}

