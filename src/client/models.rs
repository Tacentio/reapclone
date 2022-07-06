use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub ssh_url: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub login: String,
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub url: String,
    pub sha: String,
    pub node_id: String,
    pub author: Option<Author>,
}

