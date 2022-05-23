# reapclone

A CLI tool to clone all visible GitHub repositories for a given user or organisation. Repositories will be cloned into the current directory.

## Prerequisites

1. `git` on your machine must be configured for SSH authentication. `reapclone` clones repositories via their SSH URL. If you're cloning private repositories, your SSH authentication must be authorised to access these repositories.
2. Optional: Create a GitHub Personal Access Token (PAT) that has access to the organisation or user you want to access. Without this, `reapclone` will only be able to see their public repositories.
3. Optional: Export the PAT into you environment ` export GITHUB_TOKEN=<PAT>`. You should include the space in front of the export so the token doesn't end up in you shell's history.

## Installation

### MacOS

Install on MacOS via homebrew.

```
brew tap tacentio/reapclone
brew install reapclone
```

## Usage

```
reapclone 0.2.3
Program to download all GitHub repositories of an organisation/user

USAGE:
    reapclone [OPTIONS] <ORGANISATION> [GITHUB_TOKEN]

ARGS:
    <ORGANISATION>    Organisation you want to clone
    <GITHUB_TOKEN>    Optional: GitHub Personal Access Token (PAT) to use to interact with the
                      GitHub API. The tool works without this, however, it will only be able to
                      find public repos for the user/organisation [env: GITHUB_TOKEN=]

OPTIONS:
    -h, --help           Print help information
        --host <HOST>    Host to use. Default is github.com
        --list           Don't clone, just list found repositories
        --port <PORT>    Port to use. Default is 443
    -V, --version        Print version information
```

## Examples

```
reapclone -o <organisation>
reapclone -u <user>

 export GITHUB_TOKEN=<PAT>
reapclone -o <organisation>
```

## Contributing

Any Pull Requests are welcome.

## TODO

- Instead of just cloning repositories, `reapclone` could check if the repository exists locally, and instead do a `git pull`.
- Add better error handling.
  - So far `reapclone`'s error handling isn't great (I think?). I'm not sure. But I'm interested to see your thoughts and PRs.
- Add documentation for `crates.io`. Is this necessary for a binary crate?
- Figure out a way to distribute easily to various platforms (Homebrew, apt, AUR, etc).
