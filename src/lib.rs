use crate::client::github::GithubClient;
use crate::client::github::{ApiEndpoints, PathParams};
use crate::client::models::{Repo, Commit};
use crate::config::{CommandLineArgs, SubCommands};
use futures::future;
use std::error::Error;
use tokio::task;

mod client;
pub mod config;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub async fn run(cli_args: CommandLineArgs) -> Result<(), Box<dyn Error>> {
    match &cli_args.commands {
        SubCommands::Clone => clone(&cli_args).await,
        SubCommands::EnumerateUsers { max_pages }  => enumerate_users(&cli_args, *max_pages).await,
        SubCommands::ListRepos => list_repos(&cli_args).await
    }
}

pub async fn list_repos(cli_args: &CommandLineArgs) -> Result<(), Box<dyn Error>> {
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;
    let org_type = gh_client.determine_org_type(&cli_args.organisation).await?;
    let params = PathParams {
        base_url: gh_client.base_url.to_owned(),
        owner: Some(cli_args.organisation.to_owned()),
        user_type: Some(org_type),
        repo: None
    };

    let repos: Vec<Repo> = gh_client.generic_list_all_api(
        &ApiEndpoints::ListRepos, 
        &params,
        None
    ).await?;

    for repo in repos {
        println!("{}", repo.name);
    }

    Ok(())
}

pub async fn enumerate_users(cli_args: &CommandLineArgs, max_pages: Option<u32>) -> Result<(), Box<dyn Error>> {
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;
    let org_type = gh_client.determine_org_type(&cli_args.organisation).await?;

    let params = PathParams {
        base_url: gh_client.base_url.to_owned(),
        owner: Some(cli_args.organisation.to_owned()),
        user_type: Some(org_type),
        repo: None
    };

    let repos: Vec<Repo> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListRepos, &params, max_pages)
        .await?;

    for repo in repos {
        let commits = gh_client
            .generic_list_all_api::<Commit>(
                &ApiEndpoints::ListCommits,
                &PathParams {
                    base_url: gh_client.base_url.to_owned(),
                    owner: Some(cli_args.organisation.to_owned()),
                    repo: Some(repo.name.to_owned()),
                    user_type: None,
                },
                max_pages,
            )
            .await?;

        for commit in commits {
            if let Some(author) = commit.author {
                println!("{}", author.login);
            }
        }
    }

    Ok(())
}

pub async fn clone(cli_args: &CommandLineArgs) -> Result<(), Box<dyn Error>> {
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;
    let org_type = gh_client.determine_org_type(&cli_args.organisation).await?;
    let mut handles = Vec::new();
    let output_dir = match &cli_args.output_directory {
        Some(d) => d.to_owned(),
        None => String::from("./"),
    };

    println!("Searching for repositories...");

    let params = PathParams {
        base_url: gh_client.base_url.to_owned(),
        owner: Some(cli_args.organisation.to_owned()),
        user_type: Some(org_type),
        repo: None
    };

    let repos: Vec<Repo> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListRepos, &params, None)
        .await?;

        for repo in repos {
            let dir = output_dir.to_owned();
            handles.push(task::spawn(async move {
                GithubClient::clone_repo(&repo, dir).await
            }));
        }

    future::join_all(handles).await;
    Ok(())
}
