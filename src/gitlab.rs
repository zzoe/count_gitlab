use async_net::TcpStream;
use http_types::{Method, Request, Response, StatusCode, Url};
use log::{info, trace};
use serde_derive::{Deserialize, Serialize};
use util::Select;

use crate::config::ProjectId;
use crate::{CONFIG, EXECUTOR};
use async_channel::{Receiver, Sender};
use futures_lite::StreamExt;

pub async fn deal_project(s: Sender<()>, r: Receiver<()>, id: ProjectId) -> CommitLogs {
    let page = 0;
    let res = query(id, page).await;

    if res.status().eq(&StatusCode::Unauthorized) {
        panic!("Unauthorized")
    }

    trace!("x-total-pages: {:?}", res.header("X-Total-Pages"));
    let total = res.header("X-Total-Pages").map_or_else(
        || 0_u16,
        |h| {
            let v = h.get(0).expect("未获取到总条数");
            v.to_string().parse::<u16>().unwrap_or(1_u16)
        },
    );
    info!("{}总页数： {}", id, total);

    let mut tasks = Select(Vec::new());
    for page in 1..total + 1 {
        let s = s.clone();
        let r = r.clone();
        tasks.0.push(EXECUTOR.spawn(async move {
            s.send(()).await.expect("channel send fail");
            let mut res = query(id, page).await;
            info!("{}第{}页查询结束", id, page);
            trace!("res: {:?}", res);
            let mut logs: CommitLogs = res.body_json().await.expect("json解析失败");
            logs.iter_mut().for_each(|log| log.project_id = id.0);
            trace!("{:?}", logs);
            r.recv().await.expect("channel receive fail");
            logs
        }));
    }

    let mut commit_logs = CommitLogs::new();
    while let Some(mut logs) = tasks.next().await {
        commit_logs.append(&mut logs);
    }

    commit_logs
}

pub type CommitLogs = Vec<CommitLog>;

#[derive(Clone, Debug, Deserialize)]
pub struct CommitLog {
    #[serde(default = "u32::default")]
    pub project_id: u32,
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
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Serialize)]
struct Query {
    page: u16,
    per_page: u8,
    all: bool,
    with_stats: bool,
}

async fn query(id: ProjectId, page: u16) -> Response {
    let url = Url::parse(&*CONFIG.gitlab.addr)
        .expect("gitlab 地址解析失败")
        .join(&*format!("api/v4/projects/{}/repository/commits/", id.0))
        .expect("url join 失败");

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
    trace!("{}: {}", method, url.as_str());

    let mut req = Request::new(method, url);
    req.insert_header("PRIVATE-TOKEN", &*CONFIG.gitlab.token);
    req.set_query(&query).expect("设置查询条件失败");

    let addr = req
        .url()
        .socket_addrs(|| None)
        .expect("Resolve a URL’s host and port number to SocketAddr failed")
        .into_iter()
        .next()
        .expect("Resolve a URL’s host and port number to SocketAddr failed");

    let stream = TcpStream::connect(addr).await.expect("TCP连接失败");
    req.set_peer_addr(stream.peer_addr().ok());
    req.set_local_addr(stream.local_addr().ok());

    if page.ne(&0) {
        info!("开始查询{}第{}页", id, page);
    }

    async_h1::connect(stream.clone(), req)
        .await
        .expect("调用gitlab api失败")
}
