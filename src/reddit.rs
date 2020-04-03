use url;

pub enum Post {
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

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum Object {
    Listing {
        modhash: Option<String>,
        dist: u64,
        after: Option<String>,
        before: Option<String>,
        children: Vec<Object>,
    },
    #[serde(rename = "t3")]
    Link {
        subreddit: String,
        title: String,
        name: String,
    },
}

pub enum SelfPostBody {
    Text(String),
    RichtextJson(String),
}
