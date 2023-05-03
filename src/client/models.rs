use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub ssh_url: String,
    pub name: String,
    pub archived: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub login: String,
}

#[derive(Serialize, Deserialize)]
pub struct GitUser {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CommitInfo {
    pub author: Option<GitUser>,
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub url: String,
    pub sha: String,
    pub node_id: String,
    pub author: Option<Author>,
    pub commit: CommitInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ShortBranch {
    pub name: String,
}
