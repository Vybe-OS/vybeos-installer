use crate::core::installer::Installer;

#[derive(Default)]
pub struct Config {
    pub disk: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub hostname: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Config {
    pub fn validate(&self) -> Result<Installer, String> {
        Ok(Installer {
            disk: Self::device_path(self.disk.as_deref().ok_or("disk not selected")?),
            locale: self.locale.clone().ok_or("locale not specified")?,
            timezone: self.timezone.clone().ok_or("timezone not specified")?,
            hostname: self.hostname.clone().ok_or("hostname not specified")?,
            username: self.username.clone().ok_or("username not specified")?,
            password: self.password.clone().ok_or("password not specified")?,
        })
    }

    fn device_path(name: &str) -> String {
        if name.starts_with("/dev/") {
            name.to_owned()
        } else {
            format!("/dev/{name}")
        }
    }
}