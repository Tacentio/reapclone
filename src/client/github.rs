use reqwest::header;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Stdio;
use tokio::process::Command;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub ssh_url: String,
}

pub struct GithubClient {
    http_client: reqwest::Client,
}

#[derive(Clone)]
pub enum GitHubUserType {
    User,
    Organisation,
}

impl GithubClient {
    pub fn new(auth_token: Option<&str>, user_agent: &str) -> Result<GithubClient, Box<dyn Error>> {
        match auth_token {
            Some(t) => {
                let mut headers = header::HeaderMap::new();
                let token_value = format!("token {}", t);
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&token_value)?,
                );
                return Ok(GithubClient {
                    http_client: reqwest::Client::builder()
                        .user_agent(user_agent)
                        .default_headers(headers)
                        .build()?,
                });
            }
            None => {
                return Ok(GithubClient {
                    http_client: reqwest::Client::builder().user_agent(user_agent).build()?,
                })
            }
        }
    }

    /// List GitHub repos for a user or organisation.
    pub async fn list_repos(
        &self,
        user_type: GitHubUserType,
        org: &str,
        page: Option<u32>,
    ) -> Result<Vec<Repo>, Box<dyn Error>> {
        let page_number = page.unwrap_or(1);
        let url_base = match user_type {
            GitHubUserType::Organisation => format!("https://api.github.com/orgs/{}/repos", org),
            GitHubUserType::User => format!("https://api.github.com/users/{}/repos", org),
        };
        let query_string = vec![("page", format!("{}", page_number))];
        let url = Url::parse_with_params(&url_base, query_string)?;
        let r_body = self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<Vec<Repo>>()
            .await?;
        return Ok(r_body);
    }

    pub async fn list_all_repos(
        &self,
        user_type: GitHubUserType,
        org: &str,
    ) -> Result<Vec<Repo>, Box<dyn Error>> {
        let mut all_repos: Vec<Repo> = Vec::new();
        let mut page_number: u32 = 1;

        loop {
            let r_body = match user_type {
                GitHubUserType::User => {
                    self.list_repos(GitHubUserType::User, &org, Some(page_number))
                        .await?
                }
                GitHubUserType::Organisation => {
                    self.list_repos(GitHubUserType::Organisation, &org, Some(page_number))
                        .await?
                }
            };

            if r_body.is_empty() {
                break;
            }

            for repo in r_body {
                all_repos.push(repo);
            }

            page_number += 1;
        }

        Ok(all_repos)
    }

    /// Clones a GitHub repository, using it's SSH URL by spawning
    /// a child process.
    pub async fn clone_repo(repo: &Repo) {
        let mut child = Command::new("git")
            .arg("clone")
            .arg(repo.ssh_url.to_owned())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to swawn git");
        let status = child.wait().await.unwrap();

        if status.success() {
            println!("{}|SUCCESS", repo.ssh_url);
        } else {
            eprintln!("{}|FAIL", repo.ssh_url);
        }
    }
}
