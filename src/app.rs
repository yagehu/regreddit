use url;

use crate::client;
use crate::error;
use crate::reddit;
use crate::settings;

pub trait App {
    fn regreddit(
        &mut self,
        p: RegredditParams,
    ) -> error::Result<RegredditResult>;
    fn submit(&mut self, p: SubmitParams) -> error::Result<SubmitResult>;
}

pub struct AppImpl<'a> {
    client: &'a mut dyn client::Client,
}

pub struct Params<'a> {
    pub client: &'a mut dyn client::Client,
}

impl<'a> AppImpl<'a> {
    pub fn new(p: Params<'a>) -> Self {
        AppImpl { client: p.client }
    }
}

impl App for AppImpl<'_> {
    fn regreddit(
        &mut self,
        p: RegredditParams,
    ) -> error::Result<RegredditResult> {
        eprintln!("Nuking your Reddit...");

        let res = self.client.basic_auth(client::BasicAuthParams {
            credentials: p.credentials,
        })?;

        eprintln!("{:?}", res);

        let res = self.client.get_posts(client::GetPostsParams {
            username: &p.credentials.username,
        })?;

        eprintln!("{:?}", res);

        Ok(RegredditResult {})
    }

    fn submit(&mut self, p: SubmitParams) -> error::Result<SubmitResult> {
        self.client.basic_auth(client::BasicAuthParams {
            credentials: p.credentials,
        });

        match p.post_type {
            "link" => {
                let url;

                log::info!("Parsing URL...");

                match p.url {
                    Some(s) => match url::Url::parse(&s) {
                        Ok(u) => url = u,
                        Err(err) => {
                            log::error!("Failed to parse URL: {}", err);

                            return Err(error::Error::new(
                                error::ErrorKind::InvalidInput,
                                err,
                            ));
                        }
                    },
                    None => {
                        log::error!("Missing URL.");

                        return Err(error::Error::from(
                            error::ErrorKind::InvalidInput,
                        ));
                    }
                }

                log::info!("Submitting link to r/{}...", p.subreddit);

                let _ = self.client.submit(client::SubmitParams {
                    post: reddit::Post::Link {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        url,
                    },
                })?;
            }
            _ => {}
        }

        Ok(SubmitResult {})
    }
}

pub struct SubmitParams<'a> {
    pub credentials: &'a settings::Credentials,
    pub post_type: &'a str,
    pub subreddit: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

pub struct SubmitResult {}

pub struct RegredditParams<'a> {
    pub credentials: &'a settings::Credentials,
}

pub struct RegredditResult {}
