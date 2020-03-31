use std::fs;

use async_trait::async_trait;
use url;

use crate::client;
use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

#[async_trait]
pub trait App: Send {
    async fn regreddit(
        &mut self,
        p: &RegredditParams<'_>,
    ) -> Result<RegredditResult>;
    async fn submit_link(
        &mut self,
        p: &SubmitLinkParams<'_>,
    ) -> Result<SubmitLinkResult>;
    async fn submit_self_post(
        &mut self,
        p: &SubmitSelfPostParams<'_>,
    ) -> Result<SubmitSelfPostResult>;
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

#[async_trait]
impl App for AppImpl<'_> {
    async fn regreddit(
        &mut self,
        p: &RegredditParams<'_>,
    ) -> Result<RegredditResult> {
        eprintln!("Nuking your Reddit...");

        let _ = self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?;

        Ok(RegredditResult {})
    }

    async fn submit_link(
        &mut self,
        p: &SubmitLinkParams<'_>,
    ) -> Result<SubmitLinkResult> {
        log::info!("Authenticating with Reddit...");

        let _ = self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?;

        log::info!("Authentication successful.");
        log::info!("Parsing URL...");

        let url;

        match url::Url::parse(&p.url) {
            Ok(u) => url = u,
            Err(err) => {
                log::error!("Failed to parse URL: {}", err);

                return Err(Error::new(ErrorKind::InvalidInput, err));
            }
        }

        log::info!("Submitting link to r/{}...", p.subreddit);

        let _ = self
            .client
            .submit(&client::SubmitParams {
                post: reddit::Post::Link {
                    subreddit: p.subreddit.to_string(),
                    title: p.title.to_string(),
                    url,
                },
            })
            .await?;

        Ok(SubmitLinkResult {})
    }

    async fn submit_self_post(
        &mut self,
        p: &SubmitSelfPostParams<'_>,
    ) -> Result<SubmitSelfPostResult> {
        log::info!("Authenticating with Reddit...");

        let _ = self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?;

        log::info!("Authentication successful.");
        log::info!("Submitting self-post to r/{}...", p.subreddit);

        let submit_params: client::SubmitParams;

        match (p.text, p.text_file, p.richtext_json, p.richtext_json_file) {
            (Some(t), None, None, None) => {
                submit_params = client::SubmitParams {
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::Text(t.to_string()),
                    },
                }
            }
            (None, Some(f), None, None) => {
                submit_params = client::SubmitParams {
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::Text(fs::read_to_string(
                            f,
                        )?),
                    },
                }
            }
            (None, None, Some(r), None) => {
                submit_params = client::SubmitParams {
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::RichtextJson(r.to_string()),
                    },
                }
            }
            (None, None, None, Some(f)) => {
                submit_params = client::SubmitParams {
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::RichtextJson(
                            fs::read_to_string(f)?,
                        ),
                    },
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "only one input source is accepted",
                ))
            }
        }

        let _ = self.client.submit(&submit_params).await?;

        Ok(SubmitSelfPostResult {})
    }
}

pub struct SubmitLinkParams<'a> {
    pub credentials: &'a settings::Credentials,
    pub subreddit: &'a str,
    pub title: &'a str,
    pub url: &'a str,
}

pub struct SubmitLinkResult {}

pub struct SubmitSelfPostParams<'a> {
    pub credentials: &'a settings::Credentials,
    pub subreddit: &'a str,
    pub title: &'a str,
    pub text: Option<&'a str>,
    pub text_file: Option<&'a str>,
    pub richtext_json: Option<&'a str>,
    pub richtext_json_file: Option<&'a str>,
}

pub struct SubmitSelfPostResult {}

pub struct RegredditParams<'a> {
    pub credentials: &'a settings::Credentials,
}

pub struct RegredditResult {}
