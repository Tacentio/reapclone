use crate::client::github::GitHubUserType;
use crate::client::github::GithubClient;
use crate::config::CommandLineArgs;
use futures::future;
use std::error::Error;
use tokio::task;

mod client;
pub mod config;

// The GitHub API requires a User-Agent to be set when using it.
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

// TODO: Tidy this function up a bit. It works, but it seems quite ugly.
pub async fn run(cli_args: CommandLineArgs) -> Result<(), Box<dyn Error>> {
    // Currently, validation errors return the same message.
    // TODO: Look into returning different kinds of errors like done by
    // the standard library (for example, IO errors).
    cli_args.validate()?;
    let gh_client = GithubClient::new(cli_args.github_token.as_deref(), APP_USER_AGENT)?;

    if let Some(user) = cli_args.user {
        let mut handles = Vec::new();
        println!("Cloning from user {}", user);
        let repos = gh_client
            .list_all_repos(GitHubUserType::User, &user)
            .await?;
        for repo in repos {
            handles.push(task::spawn(
                async move { GithubClient::clone_repo(&repo).await },
            ));
        }
        future::join_all(handles).await;
    } else if let Some(org) = cli_args.organisation {
        let mut handles = Vec::new();
        println!("Cloning from org {}", org);
        let repos = gh_client
            .list_all_repos(GitHubUserType::Organisation, &org)
            .await?;

        for repo in repos {
            handles.push(task::spawn(
                async move { GithubClient::clone_repo(&repo).await },
            ));
        }

        future::join_all(handles).await;
    }

    Ok(())
}
