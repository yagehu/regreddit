use reqwest;

use std::collections::HashMap;
use tokio::runtime::Runtime;

use crate::error;
use crate::error::ErrorKind;
use crate::reddit;
use crate::settings;

pub trait Client {
    fn basic_auth(
        &mut self,
        p: BasicAuthParams,
    ) -> error::Result<BasicAuthResult>;
    fn get_posts(&self, p: GetPostsParams) -> error::Result<GetPostsResult>;
    fn submit(&self, p: SubmitParams) -> error::Result<SubmitResult>;
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
    fn basic_auth(
        &mut self,
        p: BasicAuthParams,
    ) -> error::Result<BasicAuthResult> {
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
                    return Err(error::Error::new(
                        ErrorKind::Authentication,
                        err,
                    ))
                }
            }

            if res.status() != reqwest::StatusCode::OK {
                eprintln!(
                    "Authentication failed with status {}.",
                    res.status(),
                );

                return Err(error::Error::from(ErrorKind::Authentication));
            }

            match res.json::<GetTokenResponse>().await {
                Ok(res) => self.access_token = res.access_token,
                Err(err) => {
                    return Err(error::Error::new(
                        ErrorKind::Authentication,
                        err,
                    ))
                }
            }

            Ok::<(), error::Error>(())
        })?;

        Ok(BasicAuthResult {})
    }

    fn get_posts(&self, p: GetPostsParams) -> error::Result<GetPostsResult> {
        self.http_client.get(&format!(
            "https://oauth.reddit.com/user/{}/submitted",
            p.username
        ));
        Ok(GetPostsResult {})
    }

    fn submit(&self, p: SubmitParams) -> error::Result<SubmitResult> {
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
                            url: url.to_string(),
                            resubmit: true,
                        })
                        .send()
                        .await
                    {
                        Ok(resp) => res = resp,
                        Err(err) => {
                            return Err(error::Error::new(
                                ErrorKind::Network,
                                err,
                            ))
                        }
                    }

                    if res.status() != reqwest::StatusCode::OK {
                        log::error!(
                            "Authentication failed with status {}.",
                            res.status()
                        );

                        return Err(error::Error::from(ErrorKind::Reddit));
                    }

                    match res.text().await {
                        Err(err) => {
                            return Err(error::Error::new(
                                ErrorKind::Network,
                                err,
                            ))
                        }
                        Ok(text) => {
                            match serde_json::from_str::<SubmitResponse>(&text)
                            {
                                Ok(res) => {
                                    if res.success == false {
                                        return Err(error::Error::from(
                                            ErrorKind::Reddit,
                                        ));
                                    }
                                }
                                Err(err) => {
                                    return Err(error::Error::new(
                                        ErrorKind::Reddit,
                                        err,
                                    ))
                                }
                            }
                        }
                    }

                    log::info!("Successfully submitted a link.");
                    Ok::<(), error::Error>(())
                })?;
            }
            _ => {}
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

#[derive(Serialize)]
struct SubmitRequest {
    #[serde(rename(serialize = "sr"))]
    subreddit: String,
    title: String,
    kind: String,
    url: String,
    resubmit: bool,
}

#[derive(Deserialize)]
struct SubmitResponse {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct GetTokenResponse {
    access_token: String,
}
