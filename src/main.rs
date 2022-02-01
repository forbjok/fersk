mod command;
mod config;
mod git;
mod util;

use anyhow::{anyhow, Context};
use clap::Parser;
use config::Config;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::git::GitRev;

#[derive(Debug, Parser)]
#[clap(name = "fersk", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(name = "generate-config", about = "Generate default configuration files")]
    GenerateConfig,

    #[clap(name = "run", about = "Run a command")]
    Run {
        #[clap(long = "branch", help = "Specify branch to check out")]
        branch: Option<String>,
        #[clap(long = "commit", help = "Specify commit to check out")]
        commit: Option<String>,
        #[clap(last = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // Initialize logging
    initialize_logging();

    let cfg = Config::from_default_location().unwrap();

    let work_root = cfg.work_path;

    let current_path = std::env::current_dir().with_context(|| "Error getting current directory")?;

    match opt.command {
        Command::GenerateConfig => {
            Config::write_default().with_context(|| "Error writing default config")?;
        }
        Command::Run { branch, commit, args } => {
            if args.is_empty() {
                return Err(anyhow!("No command specified."));
            }

            let repository_root_path =
                git::get_repository_root(current_path).with_context(|| "Not a git repository.")?;

            let repository_root_path = util::normalize_path(repository_root_path);

            // If a branch is specified, use that. Otherwise, use the branch we're currently in.
            let branch = if let Some(branch) = branch {
                GitRev::Branch(branch)
            } else if let Some(commit) = commit {
                GitRev::Commit(commit)
            } else {
                git::get_current_head(&repository_root_path).with_context(|| "Error getting current branch")?
            };

            let source_path_hash = util::hash::hash_bytes(repository_root_path.to_string_lossy().as_bytes());

            let work_path = work_root.join(source_path_hash);

            println!("Source repository: {}", repository_root_path.display());
            println!("Working directory: {}", work_path.display());
            println!("Branch: {branch}");

            let branch = match branch {
                // If it's a branch, add remote specification
                GitRev::Branch(branch) => GitRev::Branch(format!("origin/{branch}")),
                v => v,
            };

            if work_path.exists() {
                git::fetch(&work_path, "origin").with_context(|| "Error fetching repository")?;
            } else {
                std::fs::create_dir_all(&work_path)
                    .with_context(|| format!("Error creating work directory: {}", work_path.display()))?;

                git::clone(repository_root_path, &work_path).with_context(|| "Error cloning git repository")?;
            }

            // Cleanse repository
            git::cleanse(&work_path).with_context(|| "Error cleansing repository")?;

            // Check out branch in working directory
            git::checkout(&work_path, &branch).with_context(|| "Error checking out branch")?;

            // Run command
            command::exec_command(&args[0], |c| {
                c.current_dir(work_path);
                c.args(&args[1..]);
            })?;
        }
    };

    Ok(())
}

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}
