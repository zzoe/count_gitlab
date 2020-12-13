use futures_lite::future;
use serde_derive::Serialize;
use tide::{log::LevelFilter, Body, Request, Response, Result, StatusCode};

fn main() -> std::io::Result<()> {
    tide::log::with_level(LevelFilter::Debug);
    let mut app = tide::new();
    app.at("/*").get(select);
    app.at("/*").head(head);
    future::block_on(app.listen("127.0.0.1:3001"))
}

async fn head(_: Request<()>) -> Result {
    let res = Response::builder(StatusCode::Ok)
        .header("x-total-pages", "1")
        .build();
    Ok(res)
}

async fn select(_: Request<()>) -> Result {
    let mut res = Response::new(StatusCode::Ok);

    let mut logs = Vec::new();
    for i in 1..11 {
        let log = CommitLog {
            full_id: i.to_string(),
            short_id: "".to_string(),
            author_name: i.to_string(),
            author_email: "".to_string(),
            authored_date: "20201211T121212".to_string(),
            committer_name: "".to_string(),
            committer_email: "".to_string(),
            committed_date: "".to_string(),
            created_at: "".to_string(),
            title: "".to_string(),
            message: "".to_string(),
            parent_ids: vec![],
            stats: Stats {
                additions: i,
                deletions: 0,
            },
        };
        logs.push(log);
    }
    res.set_body(Body::from_json(&logs)?);
    Ok(res)
}

pub type CommitLogs = Vec<CommitLog>;

#[derive(Clone, Debug, Default, Serialize)]
pub struct CommitLog {
    #[serde(rename = "id")]
    pub full_id: String,
    pub short_id: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_date: String,
    pub created_at: String,
    pub title: String,
    pub message: String,
    pub parent_ids: Vec<String>,
    pub stats: Stats,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Stats {
    pub additions: u32,
    pub deletions: u32,
}
