use std::collections::HashMap;

use async_trait::async_trait;
use reqwest;

use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

#[async_trait]
pub trait Client: Send + Sync {
    async fn basic_auth(
        &mut self,
        p: &BasicAuthParams<'_>,
    ) -> Result<BasicAuthResult>;
    async fn get_posts(&self, p: &GetPostsParams<'_>)
        -> Result<GetPostsResult>;
    async fn submit(&self, p: &SubmitParams) -> Result<SubmitResult>;
}

pub struct ClientImpl {
    access_token: String,
    http_client: reqwest::Client,
    user_agent: String,
}

pub struct Params {
    pub user_agent: String,
}

impl ClientImpl {
    pub fn new(p: Params) -> Self {
        ClientImpl {
            access_token: "".to_string(),
            http_client: reqwest::Client::new(),
            user_agent: p.user_agent,
        }
    }
}

#[async_trait]
impl Client for ClientImpl {
    async fn basic_auth(
        &mut self,
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

        match res.json::<GetTokenResponse>().await {
            Ok(res) => self.access_token = res.access_token,
            Err(err) => return Err(Error::new(ErrorKind::Authentication, err)),
        }

        Ok(BasicAuthResult {})
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
            .header("Authorization", format!("Bearer {}", self.access_token))
            .query(&p.listing_control)
            .send()
            .await?;

        Ok(GetPostsResult {
            response: check_response::<reddit::Object>(res).await?,
        })
    }

    async fn submit(&self, p: &SubmitParams) -> Result<SubmitResult> {
        match &p.post {
            reddit::Post::Link {
                ref subreddit,
                ref title,
                ref url,
            } => {
                log::info!("Making POST request to Reddit...");

                let res;

                match self
                    .http_client
                    .post("https://oauth.reddit.com/api/submit")
                    .header("User-Agent", &self.user_agent)
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.access_token),
                    )
                    .form(&SubmitRequest {
                        subreddit,
                        title,
                        kind: "link".to_string(),
                        url: Some(&url.to_string()),
                        resubmit: true,
                        text: None,
                        richtext_json: None,
                    })
                    .send()
                    .await
                {
                    Ok(r) => res = r,
                    Err(err) => {
                        return Err(Error::new(ErrorKind::Network, err))
                    }
                }

                check_submit_response(res).await?;
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
                        request = SubmitRequest {
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
                        request = SubmitRequest {
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

                let res;

                log::info!("Making POST request to Reddit...");

                match self
                    .http_client
                    .post("https://oauth.reddit.com/api/submit")
                    .header("User-Agent", &self.user_agent)
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.access_token),
                    )
                    .form(&request)
                    .send()
                    .await
                {
                    Ok(r) => res = r,
                    Err(err) => {
                        return Err(Error::new(ErrorKind::Network, err))
                    }
                }

                check_submit_response(res).await?;
                log::info!("Successfully submitted a self-post.");
            }
        }

        Ok(SubmitResult {})
    }
}

pub struct BasicAuthParams<'a> {
    pub credentials: &'a settings::Credentials,
}

#[derive(Debug)]
pub struct BasicAuthResult {}

pub struct GetPostsParams<'a> {
    pub username: &'a str,
    pub listing_control: &'a ListingControl,
}

pub struct GetPostsResult {
    pub response: reddit::Object,
}

pub struct SubmitParams {
    pub post: reddit::Post,
}

pub struct SubmitResult {}

#[derive(Debug, Serialize)]
struct SubmitRequest<'a> {
    #[serde(rename(serialize = "sr"))]
    subreddit: &'a str,
    title: &'a str,
    kind: String,
    url: Option<&'a str>,
    resubmit: bool,
    text: Option<&'a str>,
    richtext_json: Option<&'a str>,
}

#[derive(Deserialize)]
struct SubmitResponse {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct GetTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListingControl {
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<u32>,
    pub count: Option<u32>,
    pub show: Option<ListingShow>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ListingShow {
    #[serde(rename = "all")]
    All,
}

async fn check_submit_response(res: reqwest::Response) -> Result<()> {
    if res.status() != reqwest::StatusCode::OK {
        log::error!("Authentication failed with status {}.", res.status());

        return Err(Error::from(ErrorKind::Reddit));
    }

    match res.text().await {
        Err(err) => return Err(Error::new(ErrorKind::Network, err)),
        Ok(text) => {
            log::debug!("{}", text);

            match serde_json::from_str::<SubmitResponse>(&text) {
                Ok(res) => {
                    if !res.success {
                        log::error!("Submit failed.");
                        return Err(Error::new(
                            ErrorKind::Reddit,
                            "failed to submit",
                        ));
                    }

                    log::info!("Submit OK.");
                }
                Err(err) => {
                    log::error!("Could not deserialize response");
                    return Err(Error::new(ErrorKind::Reddit, err));
                }
            }
        }
    }

    Ok(())
}

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
        Ok(res) => {
            return Ok(res);
        }
        Err(err) => {
            log::error!("Could not deserialize response");
            return Err(Error::new(ErrorKind::Reddit, err));
        }
    }
}
