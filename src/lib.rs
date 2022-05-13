use crate::client::github::GithubClient;
use crate::config::CommandLineArgs;
use futures::future;
use std::error::Error;
use tokio::task;

mod client;
pub mod config;
mod config_error;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub async fn run(cli_args: CommandLineArgs) -> Result<(), Box<dyn Error>> {
    let parsed_config = cli_args.validate()?;
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
    )?;
    let mut handles = Vec::new();
    let repos = gh_client
        .list_all_repos(parsed_config.org_type, &parsed_config.org)
        .await?;

    for repo in repos {
        handles.push(task::spawn(
            async move { GithubClient::clone_repo(&repo).await },
        ));
    }

    future::join_all(handles).await;
    Ok(())
}
