use anyhow::Result;
use log::info;

use crate::statistic::{CodeStatistics, Commits};

pub async fn save(id: usize, code_statistics: CodeStatistics) -> Result<()> {
    let date = chrono::Local::now().format("%F");
    let file_name = format!("statistics_{}_{}.csv", id, date);

    info!("开始写入文件: {}", file_name);
    smol::fs::write(file_name, generate(code_statistics)).await?;

    Ok(())
}

fn generate(code_statistics: CodeStatistics) -> String {
    const SEPARATOR: &str = ",";
    const LINE: &str = "\n";
    let mut res = Vec::new();
    let mut emails = Vec::<String>::new();

    for (someday, all_commits) in code_statistics {
        let mut msg = String::new();
        msg.push_str(&*someday);

        for email in &emails {
            let commits = all_commits.get(email);
            if commits.is_none() {
                for _ in 0..3 {
                    msg.push_str(SEPARATOR);
                }
                continue;
            }
            msg.push_str(&*commits_to_string(commits.unwrap(), SEPARATOR));
        }

        for (someone, commits) in all_commits {
            if !emails.contains(&someone) {
                emails.push(someone);
                msg.push_str(&*commits_to_string(&commits, SEPARATOR));
            }
        }

        res.push(msg);
    }

    let mut res_head1 = String::new();
    let mut res_head2 = String::new();
    res_head1.push_str("date");
    for email in emails {
        res_head1.push_str(SEPARATOR);
        res_head1.push_str(&*email);
        res_head1.push_str(SEPARATOR);
        res_head1.push_str(SEPARATOR);

        res_head2.push_str(SEPARATOR);
        res_head2.push_str("times");
        res_head2.push_str(SEPARATOR);
        res_head2.push_str("additions");
        res_head2.push_str(SEPARATOR);
        res_head2.push_str("deletions");
    }

    res.reverse();
    format!(
        "{}{}{}{}{}",
        res_head1,
        LINE,
        res_head2,
        LINE,
        res.join(LINE)
    )
}

fn commits_to_string(commits: &Commits, separator: &str) -> String {
    format!(
        "{}{}{}{}{}{}",
        separator, commits.times, separator, commits.additions, separator, commits.deletions
    )
}
