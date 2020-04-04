use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use url;

use crate::client;
use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

const LISTING_LIMIT: u32 = 50;

#[async_trait]
pub trait App: Send {
    async fn regreddit(
        &self,
        p: &RegredditParams<'_>,
    ) -> Result<RegredditResult>;
    async fn submit_link(
        &self,
        p: &SubmitLinkParams<'_>,
    ) -> Result<SubmitLinkResult>;
    async fn submit_self_post(
        &self,
        p: &SubmitSelfPostParams<'_>,
    ) -> Result<SubmitSelfPostResult>;
}

pub struct AppImpl {
    client: Arc<Box<dyn client::Client>>,
}

pub struct Params {
    pub client: Box<dyn client::Client>,
}

impl AppImpl {
    pub fn new(p: Params) -> Self {
        AppImpl {
            client: Arc::new(p.client),
        }
    }
}

#[async_trait]
impl App for AppImpl {
    async fn regreddit(
        &self,
        p: &RegredditParams<'_>,
    ) -> Result<RegredditResult> {
        log::info!("Nuking your Reddit...");

        let res = self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?;
        let access_token = res.access_token.clone();
        let limit = Some(LISTING_LIMIT);
        let mut after: Option<String> = None;
        let mut handles = Vec::new();

        loop {
            log::info!("Getting next page of posts...");

            if let reddit::Object::Listing { children, .. } = self
                .client
                .get_posts(&client::GetPostsParams {
                    access_token: &access_token,
                    username: &"trustyhardware",
                    listing_control: &reddit::ListingControl {
                        after,
                        before: None,
                        count: None,
                        limit,
                        show: None,
                    },
                })
                .await?
                .response
            {
                if children.len() == 0 {
                    break;
                }

                for post in &children {
                    if let reddit::Object::Link { name, .. } = post {
                        let access_token = access_token.clone();
                        let client = self.client.clone();
                        let name = name.clone();

                        handles.push(tokio::spawn(async move {
                            match client
                                .delete_link(&client::DeleteLinkParams {
                                    access_token: &access_token,
                                    id: &name,
                                })
                                .await
                            {
                                Ok(_res) => {
                                    log::info!("Deleted post {}.", name);
                                }
                                Err(err) => log::warn!(
                                    "Failed to delete {}: {}.",
                                    name,
                                    err
                                ),
                            }
                        }));
                    } else {
                        log::error!("Got unexpected object. Expected Link.");
                        continue;
                    }
                }

                if children.len() < LISTING_LIMIT as usize {
                    break;
                }

                if let Some(reddit::Object::Link { name, .. }) = children.last()
                {
                    after = Some(name.clone());
                } else {
                    break;
                }
            } else {
                log::error!("Got unexpected object. Expected Listing.");
                break;
            }
        }

        for handle in handles {
            let _ = handle.await;
        }

        Ok(RegredditResult {})
    }

    async fn submit_link(
        &self,
        p: &SubmitLinkParams<'_>,
    ) -> Result<SubmitLinkResult> {
        log::info!("Authenticating with Reddit...");

        let access_token = &self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?
            .access_token;
        let url = url::Url::parse(&p.url)?;

        log::info!("Authentication successful.");
        log::info!("Submitting link to r/{}...", p.subreddit);

        let _ = self
            .client
            .submit(&client::SubmitParams {
                access_token,
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
        &self,
        p: &SubmitSelfPostParams<'_>,
    ) -> Result<SubmitSelfPostResult> {
        log::info!("Authenticating with Reddit...");

        let access_token = &self
            .client
            .basic_auth(&client::BasicAuthParams {
                credentials: p.credentials,
            })
            .await?
            .access_token;
        let submit_params: client::SubmitParams;

        log::info!("Authentication successful.");
        log::info!("Submitting self-post to r/{}...", p.subreddit);

        match (p.text, p.text_file, p.richtext_json, p.richtext_json_file) {
            (Some(t), None, None, None) => {
                submit_params = client::SubmitParams {
                    access_token,
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::Text(t.to_string()),
                    },
                }
            }
            (None, Some(f), None, None) => {
                submit_params = client::SubmitParams {
                    access_token,
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
                    access_token,
                    post: reddit::Post::SelfPost {
                        subreddit: p.subreddit.to_string(),
                        title: p.title.to_string(),
                        body: reddit::SelfPostBody::RichtextJson(r.to_string()),
                    },
                }
            }
            (None, None, None, Some(f)) => {
                submit_params = client::SubmitParams {
                    access_token,
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
