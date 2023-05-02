use clap::{Parser, Subcommand};

/// Program to download all GitHub repositories of an organisation/user.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    /// Organisation you want to clone.
    pub organisation: String,
    /// Host to use. Default is github.com.
    #[clap(long)]
    pub host: Option<String>,
    /// Port to use. Default is 443.
    #[clap(long)]
    pub port: Option<String>,
    /// Directory to clone repos in to.
    #[clap(short, long)]
    pub output_directory: Option<String>,
    /// Optional: GitHub Personal Access Token (PAT) to use to interact with
    /// the GitHub API. The tool works without this, however, it will only be able
    /// to find public repos for the user/organisation.
    #[clap(env)]
    pub github_token: Option<String>,
    #[clap(subcommand)]
    pub commands: SubCommands,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    r#Clone,
    EnumerateUsers {
        /// The maximum number of pages to analyse. Can help make user
        /// enumeration quicker for large organisations.
        #[clap(short, value_parser)]
        max_pages: Option<u32>,
    },
    ListRepos,
    Repo {
        repo: String,
        #[clap(subcommand)]
        commands: RepoSubcommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum RepoSubcommands {
    Branches,
}
