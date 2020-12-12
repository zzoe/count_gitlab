use std::iter::FromIterator;

use anyhow::{anyhow, Result};
use http_types::{Method, Request, Response, Url};
use log::{debug, error, info};
use serde_derive::{Deserialize, Serialize};
use smol::net::TcpStream;
use smol::stream::StreamExt;
use util::stream_vec::StreamVec;

use crate::config::ProjectID;
use crate::CONFIG;

pub async fn deal_project(id: ProjectID) -> Result<CommitLogs> {
    let page = 0;
    let res = query(id, page).await?;
    debug!("HEAD: {:#?}", res);
    debug!("x-total-pages: {:?}", res.header("x-total-pages"));
    let total = res.header("x-total-pages").map_or_else(
        || 1_u16,
        |h| {
            let v = h.get(0).expect("未获取到总条数");
            v.to_string().parse::<u16>().unwrap_or(1_u16)
        },
    );
    info!("总页数： {}", total);

    let mut tasks = Vec::new();
    for page in 1..total + 1 {
        tasks.push(smol::spawn(async move {
            let mut res = query(id, page).await?;
            let mut logs: CommitLogs = res.body_json().await.map_err(|e| anyhow!(e))?;
            logs.iter_mut().for_each(|log| log.project_id = id.0);
            Ok(logs)
        }));
    }

    let mut commit_logs = CommitLogs::new();
    let mut tasks = StreamVec::from_iter(tasks);
    while let Some(logs_res) = tasks.next().await {
        match logs_res {
            Ok(mut logs) => commit_logs.append(&mut logs),
            e => return e,
        }
    }

    Ok(commit_logs)
}

pub type CommitLogs = Vec<CommitLog>;

#[derive(Clone, Debug, Deserialize)]
pub struct CommitLog {
    pub project_id: usize,
    #[serde(rename(deserialize = "id"))]
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

#[derive(Clone, Debug, Deserialize)]
pub struct Stats {
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Serialize)]
struct Query {
    page: u16,
    per_page: u8,
    all: bool,
    with_stats: bool,
}

async fn query(id: ProjectID, page: u16) -> Result<Response> {
    let url = Url::parse(&*CONFIG.git.addr)?
        .join(&*format!("api/v4/projects/{}/repository/commits/", id.0))?;

    let query = Query {
        page,
        per_page: 100,
        all: true,
        with_stats: true,
    };

    let method = match page {
        0 => Method::Head,
        _ => Method::Get,
    };
    info!("{}: {}", method, url.as_str());

    let mut req = Request::new(method, url);
    req.insert_header("PRIVATE-TOKEN", &*CONFIG.git.token);
    req.set_query(&query).map_err(|e| {
        error!("设置查询条件失败: {}", e);
        anyhow::anyhow!("设置查询条件失败")
    })?;

    let addr = req
        .url()
        .socket_addrs(|| match req.url().scheme() {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        })?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid gitlab address: {}", CONFIG.git.addr))?;

    let stream = TcpStream::connect(addr).await?;
    req.set_peer_addr(stream.peer_addr().ok());
    req.set_local_addr(stream.local_addr().ok());

    Ok(async_h1::connect(stream.clone(), req).await.unwrap())
}