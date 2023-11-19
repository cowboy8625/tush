/*
curl -L \
    -X POST \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer <YOUR-TOKEN>" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    https://api.github.com/repos/OWNER/REPO/issues \
    -d '{"title":"Found a bug","body":"I'\''m having a problem with this.","assignees":["octocat"],"milestone":1,"labels":["bug"]}'
*/

use std::fmt::Display;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    title: String,
    body: String,
    assignees: Vec<String>,
    labels: Vec<String>,
}

impl From<(crate::parser::Token<'_>, &str, &str, &str, &str)> for Issue {
    fn from(
        (token, owner, repo, branch, filename): (crate::parser::Token<'_>, &str, &str, &str, &str),
    ) -> Self {
        let title = format!("{:?}: {}", token.kind, token.title);
        let body = token.body(owner, repo, branch, filename);
        let assignees = vec![];
        let labels = vec![];
        Issue {
            title,
            body,
            assignees,
            labels,
        }
    }
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { title, body, .. } = self;
        write!(f, "{title}\n{body}")
    }
}

pub fn get_all_issues(owner: &str, repo: &str) -> Vec<Issue> {
    // -> Vec<Issue> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues");
    let git_token = std::env::var("TUSH_GITHUB_TOKEN").expect("TUSH_GITHUB_TOKEN not set");
    let client = Client::new();
    let request_builder = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", git_token))
        .header("User-Agent", "tush")
        .header("X-GitHub-Api-Version", "2022-11-28");
    match request_builder.send() {
        Ok(response) => {
            let text = response.text().unwrap();
            serde_json::from_str(&text).unwrap()
        }
        Err(e) => {
            println!("{:#?}", e);
            Vec::new()
        }
    }
}

pub fn post_issue(owner: &str, repo: &str, issue: &Issue) {
    let token = std::env::var("TUSH_GITHUB_TOKEN").expect("TUSH_GITHUB_TOKEN not set");

    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues");

    let client = Client::new();
    let request_builder = client
        .post(&url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "tush")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(issue);

    match request_builder.send() {
        Ok(response) => {
            println!("Posted issue. {:?}", response.text());
        }
        Err(e) => {
            println!("Failed to post issue. {:?}", e);
        }
    }
}
