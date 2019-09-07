use askama::Template;

#[derive(Template)]
#[template(path = "seed.html")]
pub struct SeedTemplate<'a> {
    pub seed: &'a str,
}
