use reqwest;

use std::collections::HashMap;
use tokio::runtime::Runtime;

use crate::error::{Error, ErrorKind, Result};
use crate::reddit;
use crate::settings;

pub trait Client {
    fn basic_auth(&mut self, p: BasicAuthParams) -> Result<BasicAuthResult>;
    fn get_posts(&self, p: GetPostsParams) -> Result<GetPostsResult>;
    fn submit(&self, p: SubmitParams) -> Result<SubmitResult>;
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

impl Client for ClientImpl {
    fn basic_auth(&mut self, p: BasicAuthParams) -> Result<BasicAuthResult> {
        let mut form = HashMap::new();
        form.insert("grant_type", "password");
        form.insert("username", &p.credentials.username);
        form.insert("password", &p.credentials.password);

        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let res;

            match self
                .http_client
                .post("https://www.reddit.com/api/v1/access_token")
                .header("User-Agent", &self.user_agent)
                .form(&form)
                .basic_auth(
                    &p.credentials.client_id,
                    Some(&p.credentials.secret),
                )
                .send()
                .await
            {
                Ok(resp) => res = resp,
                Err(err) => {
                    return Err(Error::new(ErrorKind::Authentication, err))
                }
            }

            if res.status() != reqwest::StatusCode::OK {
                eprintln!(
                    "Authentication failed with status {}.",
                    res.status(),
                );

                return Err(Error::from(ErrorKind::Authentication));
            }

            match res.json::<GetTokenResponse>().await {
                Ok(res) => self.access_token = res.access_token,
                Err(err) => {
                    return Err(Error::new(ErrorKind::Authentication, err))
                }
            }

            Ok::<(), Error>(())
        })?;

        Ok(BasicAuthResult {})
    }

    fn get_posts(&self, p: GetPostsParams) -> Result<GetPostsResult> {
        self.http_client.get(&format!(
            "https://oauth.reddit.com/user/{}/submitted",
            p.username
        ));
        Ok(GetPostsResult {})
    }

    fn submit(&self, p: SubmitParams) -> Result<SubmitResult> {
        let mut rt = Runtime::new().unwrap();

        match p.post {
            reddit::Post::Link {
                subreddit,
                title,
                url,
            } => {
                log::info!("Making POST request to Reddit...");

                rt.block_on(async {
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
                            url: Some(url.to_string()),
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

                    Ok::<(), Error>(())
                })?;
            }
            reddit::Post::SelfPost {
                subreddit,
                title,
                body,
            } => {
                let request;

                match body {
                    reddit::SelfPostBody::Text(text) => {
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
                    reddit::SelfPostBody::RichtextJson(richtext_json) => {
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

                rt.block_on(async {
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

                    Ok(())
                })?;
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
    pub username: &'a String,
}

#[derive(Debug)]
pub struct GetPostsResult {}

pub struct SubmitParams {
    pub post: reddit::Post,
}

pub struct SubmitResult {}

#[derive(Debug, Serialize)]
struct SubmitRequest {
    #[serde(rename(serialize = "sr"))]
    subreddit: String,
    title: String,
    kind: String,
    url: Option<String>,
    resubmit: bool,
    text: Option<String>,
    richtext_json: Option<String>,
}

#[derive(Deserialize)]
struct SubmitResponse {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct GetTokenResponse {
    access_token: String,
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
