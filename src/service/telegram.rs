use telegram_bot::*;
use tokio_core::reactor::Core;

use std::env;

pub struct TelegramService {
    pub api: Api
}

impl TelegramService {
    pub fn new(core: &Core) -> Self {
        let token = env::var("TELEGRAM_BOT_KEY").expect("TELEGRAM_BOT_KEY not set");
        TelegramService {
            api: Api::configure(token).build(core.handle()).unwrap()
        }
    }

}
