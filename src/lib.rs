use crate::client::github::GithubClient;
use crate::client::github::{ApiEndpoints, PathParamsBuilder};
use crate::client::models::{Commit, Repo, ShortBranch};
use crate::config::{CommandLineArgs, RepoSubcommands, SubCommands};
use futures::future;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

mod client;
pub mod config;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub async fn run(cli_args: CommandLineArgs) -> Result<(), Box<dyn Error>> {
    match &cli_args.commands {
        SubCommands::Clone => clone(&cli_args).await,
        SubCommands::EnumerateUsers { max_pages } => enumerate_users(&cli_args, *max_pages).await,
        SubCommands::ListRepos => list_repos(&cli_args).await,
        SubCommands::Repo { repo, commands } => handle_repo(&cli_args, &repo, &commands).await,
    }
}

pub async fn handle_repo(
    cli_args: &CommandLineArgs,
    repo: &str,
    commands: &RepoSubcommands,
) -> Result<(), Box<dyn Error>> {
    match commands {
        RepoSubcommands::Branches => list_branches(&cli_args, repo).await?,
    }
    Ok(())
}

pub async fn list_branches(cli_args: &CommandLineArgs, repo: &str) -> Result<(), Box<dyn Error>> {
    //println!("Getting branches...");
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;

    let params = PathParamsBuilder::new()
        .base_url(&gh_client.base_url)
        .owner(&cli_args.organisation)
        .repo(&repo)
        .build();

    let branches: Vec<ShortBranch> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListBranches, &params, None)
        .await?;
    for b in branches {
        println!("{}", b.name);
    }
    Ok(())
}

pub async fn list_repos(cli_args: &CommandLineArgs) -> Result<(), Box<dyn Error>> {
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;
    let org_type = gh_client.determine_org_type(&cli_args.organisation).await?;

    let params = PathParamsBuilder::new()
        .base_url(&gh_client.base_url)
        .owner(&cli_args.organisation)
        .user_type(org_type)
        .build();

    let repos: Vec<Repo> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListRepos, &params, None)
        .await?;

    for repo in repos {
        println!("{}", repo.name);
    }

    Ok(())
}

pub async fn enumerate_users(
    cli_args: &CommandLineArgs,
    max_pages: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    let gh_client = GithubClient::new(
        cli_args.github_token.as_deref(),
        APP_USER_AGENT,
        cli_args.host.as_deref(),
        cli_args.port.as_deref(),
    )?;
    let org_type = gh_client.determine_org_type(&cli_args.organisation).await?;

    let params = PathParamsBuilder::new()
        .base_url(&gh_client.base_url)
        .owner(&cli_args.organisation)
        .user_type(org_type)
        .build();

    let repos: Vec<Repo> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListRepos, &params, max_pages)
        .await?;

    for repo in repos {
        let params = PathParamsBuilder::new()
            .base_url(&gh_client.base_url)
            .owner(&cli_args.organisation)
            .repo(&repo.name)
            .build();

        let commits = gh_client
            .generic_list_all_api::<Commit>(&ApiEndpoints::ListCommits, &params, max_pages)
            .await?;

        for commit in commits {
            if let Some(author) = commit.author {
                if let Some(sub_author) = commit.commit.author {
                    println!("{},{}", author.login, sub_author.email);
                } else {
                    println!("{},", author.login);
                }
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

    let sem = Arc::new(Semaphore::new(20));

    let params = PathParamsBuilder::new()
        .base_url(&gh_client.base_url)
        .owner(&cli_args.organisation)
        .user_type(org_type)
        .build();

    let repos: Vec<Repo> = gh_client
        .generic_list_all_api(&ApiEndpoints::ListRepos, &params, None)
        .await?;

    for repo in repos {
        if cli_args.ignore_archived && repo.archived {
            continue;
        }
        let dir = output_dir.to_owned();
        let permit = sem.clone().acquire_owned().await.unwrap();
        handles.push(task::spawn(async move {
            GithubClient::clone_repo(&repo, dir).await;
            drop(permit);
        }));
    }

    future::join_all(handles).await;
    Ok(())
}
