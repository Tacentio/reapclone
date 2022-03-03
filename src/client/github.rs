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

    /// Lists a GitHub user's repositories.
    pub async fn get_user_repos(
        &self,
        user: &str,
        page: Option<u32>,
    ) -> Result<Vec<Repo>, Box<dyn Error>> {
        let page_number = page.unwrap_or(1);
        let url_base = format!("https://api.github.com/users/{}/repos", user);
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
    /// Lists a GitHub organisation's repositories.
    pub async fn get_org_repos(
        &self,
        org: &str,
        page: Option<u32>,
    ) -> Result<Vec<Repo>, Box<dyn Error>> {
        let page_number = page.unwrap_or(1);
        let url_base = format!("https://api.github.com/orgs/{}/repos", org);
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
    
    /// Fetches all of an organisations repositories by looping over the page numbers
    /// until an empty Array is returned. This is done synchronously to avoid hitting
    /// GitHub rate limit. Perhaps in the future this can be done async while still 
    /// paying respects to the rate limit.
    pub async fn get_all_org_repos(&self, org: &str) -> Result<Vec<Repo>, Box<dyn Error>> {
        let mut all_repos: Vec<Repo> = Vec::new();
        let mut page_number: u32 = 1;

        loop {
            let r_body = self.get_org_repos(&org, Some(page_number)).await?;

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

    pub async fn get_all_user_repos(&self, user: &str) -> Result<Vec<Repo>, Box<dyn Error>> {
        let mut all_repos: Vec<Repo> = Vec::new();
        let mut page_number: u32 = 1;

        loop {
            let r_body: Vec<Repo> = self.get_user_repos(&user, Some(page_number)).await?;

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
