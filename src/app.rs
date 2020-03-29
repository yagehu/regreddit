use std::fs;

use url;

use crate::client;
use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

pub trait App {
    fn regreddit(&mut self, p: RegredditParams) -> Result<RegredditResult>;
    fn submit_link(&mut self, p: SubmitLinkParams) -> Result<SubmitLinkResult>;
    fn submit_self_post(
        &mut self,
        p: SubmitSelfPostParams,
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

impl App for AppImpl<'_> {
    fn regreddit(&mut self, p: RegredditParams) -> Result<RegredditResult> {
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

    fn submit_link(&mut self, p: SubmitLinkParams) -> Result<SubmitLinkResult> {
        log::info!("Authenticating with Reddit...");

        let _ = self.client.basic_auth(client::BasicAuthParams {
            credentials: p.credentials,
        })?;

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

        let _ = self.client.submit(client::SubmitParams {
            post: reddit::Post::Link {
                subreddit: p.subreddit.to_string(),
                title: p.title.to_string(),
                url,
            },
        })?;

        Ok(SubmitLinkResult {})
    }

    fn submit_self_post(
        &mut self,
        p: SubmitSelfPostParams,
    ) -> Result<SubmitSelfPostResult> {
        log::info!("Authenticating with Reddit...");

        let _ = self.client.basic_auth(client::BasicAuthParams {
            credentials: p.credentials,
        })?;

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

        let _ = self.client.submit(submit_params)?;

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
