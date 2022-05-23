use reqwest::header;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process::Stdio;
use tokio::process::Command;
use url::Url;

use crate::client::github_error::{GitHubError, GitHubErrorKind};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub ssh_url: String,
}

pub struct GithubClient {
    http_client: reqwest::Client,
    base_url: String,
}

#[derive(Clone, Debug)]
pub enum GitHubUserType {
    User,
    Organisation,
}

impl GithubClient {
    /// Creates a new GithubClient.
    pub fn new(
        auth_token: Option<&str>,
        user_agent: &str,
        host: Option<&str>,
        port: Option<&str>,
    ) -> Result<GithubClient, Box<dyn Error>> {
        let t_host = match host {
            Some(h) => h.to_owned(),
            None => String::from("api.github.com"),
        };

        let t_base_path = match host {
            Some(_h) => String::from("/api/v3"),
            None => String::from(""),
        };

        let t_port = match port {
            Some(p) => format!(":{}", p.to_owned()),
            None => String::from(""),
        };

        let t_base_url = format!("https://{t_host}{t_port}{t_base_path}");

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
                    base_url: t_base_url,
                });
            }
            None => {
                return Ok(GithubClient {
                    http_client: reqwest::Client::builder().user_agent(user_agent).build()?,
                    base_url: t_base_url,
                })
            }
        }
    }

    pub async fn determine_org_type(&self, org: &str) -> Result<GitHubUserType, Box<dyn Error>> {
        if let Ok(_response) = self
            .list_repos(GitHubUserType::Organisation, org, Some(1))
            .await
        {
            return Ok(GitHubUserType::Organisation);
        } else if let Ok(_response) = self.list_repos(GitHubUserType::User, org, Some(1)).await {
            return Ok(GitHubUserType::User);
        }
        Err(Box::new(GitHubError::new(GitHubErrorKind::NotFound)))
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
            GitHubUserType::Organisation => format!("{}/orgs/{}/repos", &self.base_url, org),
            GitHubUserType::User => format!("{}/users/{}/repos", &self.base_url, org),
        };
        let query_string = vec![("page", format!("{}", page_number))];
        let url = Url::parse_with_params(&url_base, query_string)?;
        let response = self.http_client.get(url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(Box::new(GitHubError::new(GitHubErrorKind::NotFound)));
        }

        if response.status().as_u16() == 403 || response.status().as_u16() == 401 {
            return Err(Box::new(GitHubError::new(GitHubErrorKind::Unauthorized)));
        }

        let r_body = response.json::<Vec<Repo>>().await?;
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
            let r_body = self
                .list_repos(user_type.to_owned(), &org, Some(page_number))
                .await?;

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
            println!("{}|\x1B[32mSUCCESS\x1B[0m", repo.ssh_url);
        } else {
            eprintln!("{}|\x1B[31mFAIL\x1B[0m", repo.ssh_url);
        }
    }
}
