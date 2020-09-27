use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use http_types::{Method, Request, Response, Url};
use log::{debug, error, info};
use serde_derive::{Deserialize, Serialize};
use smol::channel::{unbounded, Sender};
use smol::net::TcpStream;

use crate::CONFIG;

pub async fn run(id: usize) -> Result<CodeStatistics> {
    let page = 0;
    let res = commits(id, page).await?;
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

    let (s, r) = unbounded();
    let mut tasks = vec![];
    for page in 1..=total {
        let s = s.clone();
        tasks.push(smol::spawn(count(s, id, page)));
    }

    smol::spawn(async {
        for task in tasks {
            if let Err(e) = task.await {
                error!("count失败: {}", e)
            }
        }
        drop(s)
    })
    .detach();

    let mut statistics = CodeStatistics::new();
    while let Ok(c) = r.recv().await {
        for (date, day_commits) in c {
            match statistics.get_mut(&date) {
                Some(all_commits) => {
                    for (email, commits) in day_commits {
                        match all_commits.get_mut(&email) {
                            Some(someone_commits) => {
                                someone_commits.times += commits.times;
                                someone_commits.additions += commits.additions;
                                someone_commits.deletions += commits.deletions;
                            }
                            None => {
                                all_commits.insert(email, commits);
                            }
                        }
                    }
                }
                None => {
                    statistics.insert(date, day_commits);
                }
            };
        }
    }

    Ok(statistics)
}

// <日期: <邮箱: Commits>>
pub type CodeStatistics = BTreeMap<String, HashMap<String, Commits>>;

#[derive(Debug)]
pub struct Commits {
    pub times: u8,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Deserialize)]
struct Record {
    parent_ids: Vec<String>,
    author_email: String,
    created_at: String,
    stats: Stats,
}

#[derive(Debug, Deserialize)]
struct Stats {
    additions: usize,
    deletions: usize,
}

async fn count(s: Sender<CodeStatistics>, id: usize, page: u16) -> Result<()> {
    let mut res = commits(id, page).await?;
    let records: Vec<Record> = res.body_json().await.map_err(|e| {
        error!("解析gitlab响应报错: {}", e);
        anyhow::anyhow!("解析gitlab响应报错")
    })?;

    let mut statistics = CodeStatistics::new();
    for record in records {
        if record.parent_ids.len() > 1 {
            continue;
        }

        let date = record.created_at.get(..10).unwrap_or(&*record.created_at);
        match statistics.get_mut(date) {
            None => {
                let mut someone = HashMap::new();
                someone.insert(
                    record.author_email,
                    Commits {
                        times: 1,
                        additions: record.stats.additions,
                        deletions: record.stats.deletions,
                    },
                );
                statistics.insert(date.into(), someone);
            }
            Some(all_commits) => match all_commits.get_mut(&*record.author_email) {
                None => {
                    all_commits.insert(
                        record.author_email,
                        Commits {
                            times: 1,
                            additions: record.stats.additions,
                            deletions: record.stats.deletions,
                        },
                    );
                }
                Some(commits) => {
                    commits.times += 1;
                    commits.additions += record.stats.additions;
                    commits.deletions += record.stats.deletions;
                }
            },
        }
    }

    s.send(statistics).await?;
    Ok(())
}

#[derive(Serialize)]
struct Query {
    page: u16,
    per_page: u8,
    all: bool,
    with_stats: bool,
}

async fn commits(id: usize, page: u16) -> Result<Response> {
    let url = Url::parse(&*CONFIG.git.addr)?
        .join("api/v4/projects/")?
        .join(&*format!("{}/", id))?
        .join("repository/commits/")?;

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
    Ok(async_h1::connect(stream.clone(), req).await.map_err(|e| {
        error!("调用gitlab取commits数据失败: {}", e);
        anyhow::anyhow!("调用gitlab取commits数据失败")
    })?)
}
