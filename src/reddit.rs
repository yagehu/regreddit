use url;

#[derive(Deserialize, Serialize)]
pub enum Post {
    Link {
        subreddit: String,
        title: String,
        url: url::Url,
    },
    SelfPost {
        subreddit: String,
        title: String,
    },
}
