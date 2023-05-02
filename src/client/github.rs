use reqwest::header;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::process::Stdio;
use tokio::process::Command;
use url::Url;

use crate::client::github_error::{GitHubError, GitHubErrorKind};
use crate::client::models::Repo;
use crate::client::url::build_url;

pub enum ApiEndpoints {
    ListCommits,
    ListRepos,
    ListBranches,
}

pub struct PathParams {
    pub base_url: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub user_type: Option<GitHubUserType>,
}

pub struct PathParamsBuilder {
    pub base_url: String,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub user_type: Option<GitHubUserType>,
}

impl PathParamsBuilder {
    pub fn new() -> PathParamsBuilder {
        PathParamsBuilder {
            base_url: String::from(""),
            owner: None,
            repo: None,
            user_type: None,
        }
    }

    pub fn base_url(mut self, base_url: &str) -> PathParamsBuilder {
        self.base_url = base_url.to_owned();
        self
    }

    pub fn owner(mut self, owner: &str) -> PathParamsBuilder {
        self.owner = Some(owner.to_owned());
        self
    }

    pub fn repo(mut self, repo: &str) -> PathParamsBuilder {
        self.repo = Some(repo.to_owned());
        self
    }

    pub fn user_type(mut self, user_type: GitHubUserType) -> PathParamsBuilder {
        self.user_type = Some(user_type);
        self
    }

    pub fn build(self) -> PathParams {
        PathParams {
            base_url: self.base_url,
            owner: self.owner,
            repo: self.repo,
            user_type: self.user_type,
        }
    }
}

pub struct GithubClient {
    http_client: reqwest::Client,
    pub base_url: String,
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
            .generic_list_api::<Repo>(
                &ApiEndpoints::ListRepos,
                &PathParams {
                    base_url: self.base_url.to_owned(),
                    owner: Some(org.to_owned()),
                    repo: None,
                    user_type: Some(GitHubUserType::Organisation),
                },
                None,
                None,
            )
            .await
        {
            return Ok(GitHubUserType::Organisation);
        } else if let Ok(_response) = self
            .generic_list_api::<Repo>(
                &ApiEndpoints::ListRepos,
                &PathParams {
                    base_url: self.base_url.to_owned(),
                    owner: Some(org.to_owned()),
                    repo: None,
                    user_type: Some(GitHubUserType::User),
                },
                None,
                None,
            )
            .await
        {
            return Ok(GitHubUserType::User);
        }

        Err(Box::new(GitHubError::new(GitHubErrorKind::NotFound)))
    }

    pub async fn generic_list_api<T>(
        &self,
        api_endpoint: &ApiEndpoints,
        params: &PathParams,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Vec<T>, Box<dyn Error>>
    where
        T: 'static + DeserializeOwned,
    {
        let page_number = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(30);
        let query_string = vec![
            ("page", format!("{}", page_number)),
            ("per_page", format!("{}", per_page)),
        ];
        if let Ok(url_base) = build_url(api_endpoint, params) {
            let url = Url::parse_with_params(&url_base, query_string)?;
            let response = self.http_client.get(url).send().await?;
            let response_body = response.json::<Vec<T>>().await?;
            return Ok(response_body);
        } else {
            return Err(Box::new(GitHubError::new(GitHubErrorKind::NotFound)));
        }
    }

    pub async fn generic_list_all_api<T>(
        &self,
        api_endpoint: &ApiEndpoints,
        params: &PathParams,
        max_pages: Option<u32>,
    ) -> Result<Vec<T>, Box<dyn Error>>
    where
        T: 'static + DeserializeOwned,
    {
        let mut all_items: Vec<T> = Vec::new();
        let mut page_number: u32 = 1;
        let max_page_number = max_pages.unwrap_or(65535);

        loop {
            if page_number >= max_page_number {
                break;
            }
            let response_body = self
                .generic_list_api::<T>(api_endpoint, params, Some(page_number), None)
                .await?;

            if response_body.is_empty() {
                break;
            }

            for item in response_body {
                all_items.push(item);
            }

            page_number += 1;
        }

        Ok(all_items)
    }

    /// Clones a GitHub repository, using it's SSH URL by spawning
    /// a child process.
    pub async fn clone_repo(repo: &Repo, directory: String) {
        let mut child = Command::new("git")
            .current_dir(directory)
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
