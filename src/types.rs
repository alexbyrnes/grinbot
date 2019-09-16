use reqwest::Client;

/// Global application context.
#[derive(Debug, Clone)]
pub struct Context {
    pub http_client: Client,
    pub wallet_dir: String,
    pub wallet_password: String,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            http_client: reqwest::Client::new(),
            wallet_dir: String::default(),
            wallet_password: String::default(),
        }
    }
}
