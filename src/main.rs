use clap::Parser;
use reapclone::config::CommandLineArgs;

#[tokio::main]
async fn main() {
    let cli_args = CommandLineArgs::parse();

    if let Err(e) = reapclone::run(cli_args).await {
        eprintln!("Application error: {}", e);
        return;
    }
}
