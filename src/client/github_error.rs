use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GitHubErrorKind {
    Unauthorized,
    NotFound,
}

#[derive(Debug)]
pub struct GitHubError {
    pub kind: GitHubErrorKind,
}

impl GitHubError {
    pub fn new(kind: GitHubErrorKind) -> GitHubError {
        GitHubError { kind }
    }
}

impl GitHubErrorKind {
    pub fn as_str(&self) -> &'static str {
        use GitHubErrorKind::*;
        match *self {
            Unauthorized => "unauthorized",
            NotFound => "not found",
        }
    }
}

impl fmt::Display for GitHubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GitHubError: {}", self.kind.as_str())
    }
}

impl Error for GitHubError {}
