use futures::stream::Stream;
use grinbot_core::controller::dispatch::{
    get_action, get_command, get_new_ui, screen_reducer, tokenize_command,
};
use grinbot_core::controller::types::{LoggableState, Screen, State};
use grinbot_core::types::Context;
use redux_rs::{Store, Subscription};
use telegram_bot::Api;
use tokio_core::reactor::Core;

pub struct TelegramService {}

impl TelegramService {
    pub fn new() -> Self {
        TelegramService {}
    }

    pub fn start(
        self,
        username: String,
        wallet_dir: String,
        owner_endpoint: String,
        wallet_password: String,
        log_config: String,
        cli_command: Option<&str>,
        key: String,
    ) {
        // Logging
        log4rs::init_file(log_config, Default::default()).unwrap();
        info!("Starting Grin Bot...");
        let logging_listener: Subscription<State> = |state: &State| {
            // Log actions with a log level
            if let Some(level) = state.error_level {
                log!(level, "{:#?}", LoggableState::new(state.clone()));
            }
        };

        // Initialize reqwest and app context
        let http_client = reqwest::Client::new();
        let context = Context {
            http_client,
            wallet_dir,
            owner_endpoint,
            wallet_password,
        };

        // Initial state of the bot
        let initial_state = State {
            id: None,
            prev_screen: Screen::Home,
            screen: Screen::Home,
            message: None,
            context: context,
            error_level: None,
        };

        // The state management store
        let mut store = Store::new(screen_reducer, initial_state);

        // Log actions
        store.subscribe(logging_listener);

        let mut core = Core::new().unwrap();
        let api = Api::configure(key).build(core.handle()).unwrap();

        let future = api.stream().for_each(|update| {
            let action = get_action(update, &username);
            store.dispatch(action);
            let msg = get_new_ui(store.state());
            api.spawn(msg);
            Ok(())
        });

        // Run the command line update, if any.
        if let Some(command) = cli_command {
            let (command_type, parameters) = tokenize_command(&command);
            let action = get_command(command_type, 0, parameters);
            store.dispatch(action);
            let message = &store.state().message;
            println!("{}", message.clone().unwrap());
        } else {
            // Start main loop
            core.run(future).unwrap();
            info!("Running...");
        }
    }
}
