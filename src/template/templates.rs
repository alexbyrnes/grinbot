use askama::Template;

/// Message with post-send information.
#[derive(Template)]
#[template(path = "send-success.html")]
pub struct SendSuccessTemplate<'a> {
    pub amount: f64,
    pub fee: f64,
    pub block_height: &'a str,
    pub id: &'a str,
}

/// Message returning user's seed after wallet creation.
#[derive(Template)]
#[template(path = "seed.html")]
pub struct SeedTemplate<'a> {
    pub seed: &'a str,
}
