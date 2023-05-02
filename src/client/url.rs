use crate::client::github::{ApiEndpoints, GitHubUserType, PathParams};

pub fn build_url(endpoint: &ApiEndpoints, params: &PathParams) -> Result<String, ()> {
    use ApiEndpoints::*;
    match *endpoint {
        ListCommits => {
            if let Some(owner) = &params.owner {
                if let Some(repo) = &params.repo {
                    return Ok(format!(
                        "{}/repos/{}/{}/commits",
                        params.base_url, owner, repo
                    ));
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        ListRepos => {
            if let Some(user_type) = &params.user_type {
                if let Some(owner) = &params.owner {
                    match user_type {
                        GitHubUserType::User => {
                            return Ok(format!("{}/users/{}/repos", params.base_url, owner));
                        }
                        GitHubUserType::Organisation => {
                            return Ok(format!("{}/orgs/{}/repos", params.base_url, owner));
                        }
                    }
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }

        ListBranches => {
            let owner = match &params.owner {
                Some(o) => o,
                None => return Err(()),
            };

            let repo = match &params.repo {
                Some(r) => r,
                None => return Err(()),
            };

            return Ok(format!(
                "{}/repos/{}/{}/branches",
                &params.base_url, &owner, &repo
            ));
        }
    }
}
