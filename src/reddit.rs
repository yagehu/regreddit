#[derive(Serialize)]
pub(crate) struct DeleteRequestForm<'a> {
    pub id: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ListingControl {
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<u32>,
    pub count: Option<u32>,
    pub show: Option<ListingShow>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum ListingShow {
    #[serde(rename = "all")]
    All,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub(crate) enum Object {
    Listing {
        modhash: Option<String>,
        dist: u64,
        after: Option<String>,
        before: Option<String>,
        children: Vec<Object>,
    },
    #[serde(rename = "t1")]
    Comment {
        link_title: String,
        link_id: String,
        name: String,
    },
    #[serde(rename = "t3")]
    Link {
        subreddit: String,
        title: String,
        name: String,
    },
}

pub(crate) enum Post {
    Link {
        subreddit: String,
        title: String,
        url: url::Url,
    },
    SelfPost {
        subreddit: String,
        title: String,
        body: SelfPostBody,
    },
}

pub(crate) enum SelfPostBody {
    Text(String),
    RichtextJson(String),
}

#[derive(Deserialize)]
pub(crate) struct DeleteResponse {}

#[derive(Deserialize)]
pub(crate) struct GetTokenResponse {
    pub access_token: String,
}

#[derive(Serialize)]
pub(crate) struct SubmitRequest<'a> {
    #[serde(rename(serialize = "sr"))]
    pub subreddit: &'a str,
    pub title: &'a str,
    pub kind: String,
    pub url: Option<&'a str>,
    pub resubmit: bool,
    pub text: Option<&'a str>,
    pub richtext_json: Option<&'a str>,
}

#[derive(Deserialize)]
pub(crate) struct SubmitResponse {
    pub success: bool,
}
