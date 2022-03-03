use clap::Parser;
use std::error;
use std::fmt;

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
    /// Optional: GitHub Personal Access Token (PAT) to use to interact with
    /// the GitHub API. The tool works without this, however, it will only be able
    /// to find public repos for the user/organisation.
    #[clap(env)]
    pub github_token: Option<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    BothOrgAndUserNotSet,
    BothOrgAndUserSet,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error validationg cli args")
    }
}

impl error::Error for ValidationError {}

impl CommandLineArgs {
    // The application expects either organisation OR user to be set.
    // Not both.
    // This can likely be done in a better way, this is just the best way
    // I currently know...
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(_user) = &self.user {
            if let Some(_org) = &self.organisation {
                return Err(ValidationError::BothOrgAndUserSet);
            }
            return Ok(());
        } else if let Some(_org) = &self.organisation {
            return Ok(());
        }

        Err(ValidationError::BothOrgAndUserNotSet)
    }
}
