use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

#[async_trait]
pub trait Client: Send + Sync {
    async fn basic_auth(
        &self,
        p: &BasicAuthParams<'_>,
    ) -> Result<BasicAuthResult>;
    async fn delete_link(
        &self,
        p: &DeleteLinkParams<'_>,
    ) -> Result<DeleteLinkResult>;
    async fn get_comments(
        &self,
        p: &GetCommentsParams<'_>,
    ) -> Result<GetCommentsResult>;
    async fn get_posts(&self, p: &GetPostsParams<'_>)
        -> Result<GetPostsResult>;
    async fn submit(&self, p: &SubmitParams<'_>) -> Result<SubmitResult>;
}

pub struct ClientImpl {
    http_client: reqwest::Client,
    user_agent: String,
}

pub struct Params {
    pub user_agent: String,
}

impl ClientImpl {
    pub fn new(p: Params) -> Self {
        ClientImpl {
            http_client: reqwest::Client::new(),
            user_agent: p.user_agent,
        }
    }
}

#[async_trait]
impl Client for ClientImpl {
    async fn basic_auth(
        &self,
        p: &BasicAuthParams<'_>,
    ) -> Result<BasicAuthResult> {
        let mut form = HashMap::new();
        form.insert("grant_type", "password");
        form.insert("username", &p.credentials.username);
        form.insert("password", &p.credentials.password);

        let res;

        match self
            .http_client
            .post("https://www.reddit.com/api/v1/access_token")
            .header("User-Agent", &self.user_agent)
            .form(&form)
            .basic_auth(&p.credentials.client_id, Some(&p.credentials.secret))
            .send()
            .await
        {
            Ok(resp) => res = resp,
            Err(err) => return Err(Error::new(ErrorKind::Authentication, err)),
        }

        if res.status() != reqwest::StatusCode::OK {
            eprintln!("Authentication failed with status {}.", res.status());

            return Err(Error::from(ErrorKind::Authentication));
        }

        match res.json::<reddit::GetTokenResponse>().await {
            Ok(res) => Ok(BasicAuthResult {
                access_token: res.access_token,
            }),
            Err(err) => Err(Error::new(ErrorKind::Authentication, err)),
        }
    }

    async fn delete_link(
        &self,
        p: &DeleteLinkParams<'_>,
    ) -> Result<DeleteLinkResult> {
        log::debug!("Deleting link...");

        let res = self
            .http_client
            .post("https://oauth.reddit.com/api/del")
            .header("User-Agent", &self.user_agent)
            .header("Authorization", format!("Bearer {}", p.access_token))
            .form(&reddit::DeleteRequestForm { id: p.id })
            .send()
            .await?;
        let _res = check_response::<reddit::DeleteResponse>(res).await?;

        Ok(DeleteLinkResult {})
    }

    async fn get_comments(
        &self,
        p: &GetCommentsParams<'_>,
    ) -> Result<GetCommentsResult> {
        log::debug!("Getting comments...");

        let res = self
            .http_client
            .get(&format!(
                "https://oauth.reddit.com/user/{}/comments",
                p.username
            ))
            .header("User-Agent", &self.user_agent)
            .header("Authorization", format!("Bearer {}", p.access_token))
            .query(&p.listing_control)
            .send()
            .await?;

        Ok(GetCommentsResult {
            response: check_response::<reddit::Object>(res).await?,
        })
    }

    async fn get_posts(
        &self,
        p: &GetPostsParams<'_>,
    ) -> Result<GetPostsResult> {
        log::debug!("Getting posts...");

        let res = self
            .http_client
            .get(&format!(
                "https://oauth.reddit.com/user/{}/submitted",
                p.username,
            ))
            .header("User-Agent", &self.user_agent)
            .header("Authorization", format!("Bearer {}", p.access_token))
            .query(&p.listing_control)
            .send()
            .await?;

        Ok(GetPostsResult {
            response: check_response::<reddit::Object>(res).await?,
        })
    }

    async fn submit(&self, p: &SubmitParams<'_>) -> Result<SubmitResult> {
        match &p.post {
            reddit::Post::Link {
                ref subreddit,
                ref title,
                ref url,
            } => {
                log::info!("Making POST request to Reddit...");

                let res = self
                    .http_client
                    .post("https://oauth.reddit.com/api/submit")
                    .header("User-Agent", &self.user_agent)
                    .header(
                        "Authorization",
                        format!("Bearer {}", p.access_token),
                    )
                    .form(&reddit::SubmitRequest {
                        subreddit,
                        title,
                        kind: "link".to_string(),
                        url: Some(&url.to_string()),
                        resubmit: true,
                        text: None,
                        richtext_json: None,
                    })
                    .send()
                    .await?;
                let res = check_response::<reddit::SubmitResponse>(res).await?;

                if !res.success {
                    return Err(Error::new(
                        ErrorKind::Reddit,
                        "submit unsuccessful",
                    ));
                }

                log::info!("Successfully submitted a link.");
            }
            reddit::Post::SelfPost {
                ref subreddit,
                ref title,
                ref body,
            } => {
                let request;

                match body {
                    reddit::SelfPostBody::Text(ref text) => {
                        log::info!(r#"Building a "text" self-post request..."#);
                        request = reddit::SubmitRequest {
                            subreddit,
                            title,
                            kind: "self".to_string(),
                            url: None,
                            resubmit: true,
                            text: Some(text),
                            richtext_json: None,
                        }
                    }
                    reddit::SelfPostBody::RichtextJson(ref richtext_json) => {
                        request = reddit::SubmitRequest {
                            subreddit,
                            title,
                            kind: "self".to_string(),
                            url: None,
                            resubmit: true,
                            text: None,
                            richtext_json: Some(richtext_json),
                        }
                    }
                }

                log::debug!("Making POST request to Reddit...");

                let res = self
                    .http_client
                    .post("https://oauth.reddit.com/api/submit")
                    .header("User-Agent", &self.user_agent)
                    .header(
                        "Authorization",
                        format!("Bearer {}", p.access_token),
                    )
                    .form(&request)
                    .send()
                    .await?;
                let res = check_response::<reddit::SubmitResponse>(res).await?;

                if !res.success {
                    return Err(Error::new(
                        ErrorKind::Reddit,
                        "submit unsuccessful",
                    ));
                }

                log::debug!("Successfully submitted a self-post.");
            }
        }

        Ok(SubmitResult {})
    }
}

pub struct BasicAuthParams<'a> {
    pub credentials: &'a settings::Credentials,
}

#[derive(Debug)]
pub struct BasicAuthResult {
    pub access_token: String,
}

pub struct DeleteLinkParams<'a> {
    pub access_token: &'a str,
    pub id: &'a str,
}

pub struct DeleteLinkResult {}

pub struct GetCommentsParams<'a> {
    pub access_token: &'a str,
    pub username: &'a str,
    pub listing_control: &'a reddit::ListingControl,
}

pub struct GetCommentsResult {
    pub response: reddit::Object,
}

pub struct GetPostsParams<'a> {
    pub access_token: &'a str,
    pub username: &'a str,
    pub listing_control: &'a reddit::ListingControl,
}

pub struct GetPostsResult {
    pub response: reddit::Object,
}

pub struct SubmitParams<'a> {
    pub access_token: &'a str,
    pub post: reddit::Post,
}

pub struct SubmitResult {}

async fn check_response<T: serde::de::DeserializeOwned>(
    res: reqwest::Response,
) -> Result<T> {
    if res.status() != reqwest::StatusCode::OK {
        log::error!(
            "Reqest returned bad status {}: {}",
            res.status(),
            res.text().await?
        );

        return Err(Error::from(ErrorKind::Reddit));
    }

    let text = res.text().await?;

    log::debug!("{}", text);

    match serde_json::from_str::<T>(&text) {
        Ok(res) => Ok(res),
        Err(err) => {
            log::error!("Could not deserialize response");
            Err(Error::new(ErrorKind::Reddit, err))
        }
    }
}
