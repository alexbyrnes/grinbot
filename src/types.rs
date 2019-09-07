use reqwest::Client;

/// Global application context.
#[derive(Debug, Clone)]
pub struct Context {
    pub http_client: Client,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            http_client: reqwest::Client::new(),
        }
    }
}
