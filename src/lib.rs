use crate::client::github::GitHubUserType;
use crate::client::github::GithubClient;
use crate::config::CommandLineArgs;
use futures::future;
use std::error::Error;
use std::process;
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
    let org = match &cli_args.user {
        Some(user) => user.to_owned(),
        None => match &cli_args.organisation {
            Some(org) => org.to_owned(),
            None => {
                process::exit(1);
            }
        },
    };
    let host = match cli_args.host {
        Some(host) => host,
        None => String::from("api.github.com"),
    };
    let base_path = match cli_args.enterprise {
        true => String::from("/api/v3"),
        false => String::from(""),
    };
    let base_url = format!("https://{host}{base_path}");
    let org_type = match &cli_args.user {
        Some(_user) => GitHubUserType::User,
        None => GitHubUserType::Organisation,
    };

    let mut handles = Vec::new();
    let repos = gh_client.list_all_repos(org_type, &org, &base_url).await?;

    for repo in repos {
        handles.push(task::spawn(
            async move { GithubClient::clone_repo(&repo).await },
        ));
    }

    future::join_all(handles).await;

    Ok(())
}
