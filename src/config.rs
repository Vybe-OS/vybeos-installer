#[derive(Default)]
pub struct Config {
    pub disk: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}