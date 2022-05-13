use clap::Parser;

use crate::client::github::GitHubUserType;
use crate::config_error::{ValidationError, ValidationErrorKind};

/// Program to download all GitHub repositories of an organisation/user.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    /// User you want to clone.
    #[clap(short, long)]
    pub user: Option<String>,
    /// Organisation you want to clone.
    #[clap(short, long)]
    pub organisation: Option<String>,
    /// Host to use. Default is github.com.
    #[clap(long)]
    pub host: Option<String>,
    /// Optional: GitHub Personal Access Token (PAT) to use to interact with
    /// the GitHub API. The tool works without this, however, it will only be able
    /// to find public repos for the user/organisation.
    #[clap(env)]
    pub github_token: Option<String>,
}

pub struct ParsedConfig {
    pub org: String,
    pub org_type: GitHubUserType,
}

impl CommandLineArgs {
    pub fn validate(&self) -> Result<ParsedConfig, ValidationError> {
        if let Some(_user) = &self.user {
            if let Some(_org) = &self.organisation {
                return Err(ValidationError::new(ValidationErrorKind::BothOrgAndUserSet));
            }
            let parsed_config = ParsedConfig {
                org: _user.to_owned(),
                org_type: GitHubUserType::User,
            };
            return Ok(parsed_config);
        } else if let Some(_org) = &self.organisation {
            let parsed_config = ParsedConfig {
                org: _org.to_owned(),
                org_type: GitHubUserType::Organisation,
            };
            return Ok(parsed_config);
        }

        Err(ValidationError::new(
            ValidationErrorKind::BothOrgAndUserNotSet,
        ))
    }
}
