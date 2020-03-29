use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub client_id: String,
    pub secret: String,
    pub username: String,
    pub password: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name(".regreddit").required(true))?;

        s.try_into()
    }
}