mod command;
mod config;
mod git;
mod util;

use std::{path::PathBuf, process::Stdio};

use anyhow::{anyhow, Context};
use clap::Parser;
use config::Config;
use serde_derive::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::{
    git::{Git, GitRev},
    util::pid::PidLock,
};

const FERSK_ORIGIN: &str = "fersk-origin";

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
        #[clap(long = "path", help = "Specify repository path")]
        path: Option<PathBuf>,
        #[clap(long = "branch", help = "Specify branch to check out")]
        branch: Option<String>,
        #[clap(long = "commit", help = "Specify commit to check out")]
        commit: Option<String>,
        #[clap(long = "copy-remote", help = "Specify remote to copy to the working repository")]
        copy_remote: Option<String>,
        #[clap(last = true)]
        args: Vec<String>,

        #[clap(long = "json-out", help = "Output json information on success")]
        json_out: bool,
    },
}

#[derive(Serialize)]
struct JsonOutput {
    source_repository_path: PathBuf,
    working_repository_path: PathBuf,
    branch: String,
}

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    // Initialize logging
    initialize_logging();

    let cfg = Config::from_default_location().unwrap();

    let work_root = cfg.work_path;

    match opt.command {
        Command::GenerateConfig => {
            Config::write_default().with_context(|| "Error writing default config")?;
        }
        Command::Run {
            path,
            branch,
            commit,
            copy_remote,
            args,
            json_out,
        } => {
            if args.is_empty() {
                return Err(anyhow!("No command specified."));
            }

            let path = if let Some(path) = path {
                path
            } else {
                std::env::current_dir().with_context(|| "Error getting current directory")?
            };

            let git = Git { silent: json_out };

            // Determine repository root path
            let repository_root_path = git.get_repository_root(path).with_context(|| "Not a git repository.")?;

            // Normalize repository root path
            let repository_root_path = util::normalize_path(repository_root_path);

            let source_path_hash = util::hash::hash_bytes(repository_root_path.to_string_lossy().as_bytes());

            let pidlock_path = work_root.join(format!(".locks/{source_path_hash}.pid"));
            util::create_parent_dir(&pidlock_path).with_context(|| "Cannot create PID lock directory.")?;
            let _pidlock = PidLock::acquire(pidlock_path).with_context(|| {
                "Could not acquire PID lock. Another process is already running in this repository."
            })?;

            // If a branch is specified, use that. Otherwise, use the branch we're currently in.
            let branch = if let Some(branch) = branch {
                GitRev::Branch(branch)
            } else if let Some(commit) = commit {
                GitRev::Commit(commit)
            } else {
                git.get_current_head(&repository_root_path)
                    .with_context(|| "Error getting current branch")?
            };

            let work_path = work_root.join(source_path_hash);

            if !json_out {
                println!("Source repository: {}", repository_root_path.display());
                println!("Working directory: {}", work_path.display());
                println!("Branch: {branch}");
            }

            let branch = match branch {
                // If it's a branch, add remote specification
                GitRev::Branch(branch) => GitRev::Branch(format!("{FERSK_ORIGIN}/{branch}")),
                v => v,
            };

            if work_path.exists() {
                git.force_remote_url(&work_path, FERSK_ORIGIN, &repository_root_path)
                    .with_context(|| "Error setting Fersk remote URL")?;

                git.fetch(&work_path, FERSK_ORIGIN)
                    .with_context(|| "Error fetching repository")?;
            } else {
                std::fs::create_dir_all(&work_path)
                    .with_context(|| format!("Error creating work directory: {}", work_path.display()))?;

                git.clone(&repository_root_path, &work_path, Some(FERSK_ORIGIN))
                    .with_context(|| "Error cloning git repository")?;
            }

            if let Some(copy_remote) = copy_remote {
                let remote_url = git
                    .get_remote_url(&repository_root_path, &copy_remote)
                    .with_context(|| "Error getting copy remote URL")?;

                git.force_remote_url(&work_path, &copy_remote, &remote_url)
                    .with_context(|| "Error setting copy remote URL")?;
            }

            // Cleanse repository
            git.cleanse(&work_path).with_context(|| "Error cleansing repository")?;

            // Check out branch in working directory
            git.checkout(&work_path, &branch)
                .with_context(|| "Error checking out branch")?;

            // Run command
            command::exec_command(&args[0], |c| {
                if json_out {
                    c.stdout(Stdio::null());
                }

                c.current_dir(&work_path);
                c.args(&args[1..]);
            })?;

            if json_out {
                let output = JsonOutput {
                    source_repository_path: repository_root_path,
                    working_repository_path: work_path,
                    branch: branch.to_string(),
                };

                let stdio = std::io::stdout();
                serde_json::to_writer_pretty(stdio.lock(), &output)?;
            }
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
