use askama::Template;

#[derive(Template)]
#[template(path = "send-success.html")]
pub struct SendSuccessTemplate<'a> {
    pub amount: f64,
    pub fee: f64,
    pub block_height: &'a str,
    pub id: &'a str,
}
