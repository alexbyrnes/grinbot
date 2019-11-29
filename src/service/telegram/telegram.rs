use telegram_bot::*;
use tokio_core::reactor::Core;

pub struct TelegramService {
    pub api: Api,
}

impl TelegramService {
    pub fn new(core: &Core, key: String) -> Self {
        TelegramService {
            api: Api::configure(key).build(core.handle()).unwrap(),
        }
    }
}
