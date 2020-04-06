use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Credentials {
    pub client_id: String,
    pub secret: String,
    pub username: String,
    pub password: String,
}

impl Settings {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name(".regreddit").required(true))?;

        s.try_into()
    }
}
