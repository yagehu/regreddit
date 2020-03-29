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

pub enum SelfPostBody {
    Text(String),
    RichtextJson(String),
}
