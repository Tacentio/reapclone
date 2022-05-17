use clap::Parser;
/// Program to download all GitHub repositories of an organisation/user.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CommandLineArgs {
    /// Organisation you want to clone.
    pub organisation: String,
    /// Host to use. Default is github.com.
    #[clap(long)]
    pub host: Option<String>,
    /// Optional: GitHub Personal Access Token (PAT) to use to interact with
    /// the GitHub API. The tool works without this, however, it will only be able
    /// to find public repos for the user/organisation.
    #[clap(env)]
    pub github_token: Option<String>,
}
