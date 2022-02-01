use std::ffi::OsStr;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("error executing git")]
    Execute,
    #[error("unknown error")]
    Unknown(Option<i32>),
}

#[derive(Clone)]
pub enum GitRev {
    Branch(String),
    Commit(String),
}

impl AsRef<str> for GitRev {
    fn as_ref(&self) -> &str {
        match self {
            Self::Branch(v) => v,
            Self::Commit(v) => v,
        }
    }
}

impl Display for GitRev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Branch(v) => v,
            Self::Commit(v) => v,
        };

        f.write_str(s)
    }
}

/// Cleanse repository
pub fn cleanse(path: impl AsRef<Path>) -> Result<(), GitError> {
    exec_git(|c| {
        c.current_dir(&path);

        c.args(&["reset", "--hard"]);
    })?;

    exec_git(|c| {
        c.current_dir(&path);

        c.args(&["clean", "-fdx"]);
    })?;

    Ok(())
}

/// Check out branch in repository
pub fn checkout<B>(path: impl AsRef<Path>, rev: B) -> Result<(), GitError>
where
    B: AsRef<str>,
{
    exec_git(|c| {
        c.current_dir(&path);

        c.args(&["checkout", rev.as_ref()]);
    })?;

    Ok(())
}

/// Clone repository
pub fn clone(source: impl AsRef<OsStr>, destination: impl AsRef<Path>) -> Result<(), GitError> {
    exec_git(|c| {
        c.arg("clone");
        c.arg(source);
        c.arg(destination.as_ref());
    })?;

    Ok(())
}

/// Fetch repository
pub fn fetch(path: impl AsRef<Path>, remote_name: &str) -> Result<(), GitError> {
    exec_git(|c| {
        c.current_dir(path);

        c.arg("fetch");
        c.arg(remote_name);
    })?;

    Ok(())
}

/// Get root path of repository
pub fn get_repository_root(path: impl AsRef<Path>) -> Result<PathBuf, GitError> {
    match exec_git_output(|c| {
        c.current_dir(path);

        c.args(&["rev-parse", "--show-toplevel"]);
    }) {
        Ok(output) => Ok(PathBuf::from_str(String::from_utf8_lossy(&output.stdout).trim_end()).unwrap()),
        Err(err) => Err(err),
    }
}

/// Get current branch or commit hash
pub fn get_current_head(path: impl AsRef<Path>) -> Result<GitRev, GitError> {
    let output = exec_git_output(|c| {
        c.current_dir(&path);

        c.args(&["rev-parse", "--abbrev-ref", "HEAD"]);
    })?;

    let out = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    if out != "HEAD" {
        return Ok(GitRev::Branch(out));
    }

    let output = exec_git_output(|c| {
        c.current_dir(&path);

        c.args(&["rev-parse", "HEAD"]);
    })?;

    Ok(GitRev::Commit(
        String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
    ))
}

/// Execute git command and get status
fn exec_git(f: impl FnOnce(&mut Command)) -> Result<(), GitError> {
    let mut command = Command::new("git");

    f(&mut command);

    // Execute command
    let status = command.status().map_err(|_| GitError::Execute)?;

    if !status.success() {
        return Err(GitError::Unknown(status.code()));
    }

    Ok(())
}

/// Execute git command and get output
fn exec_git_output(f: impl FnOnce(&mut Command)) -> Result<Output, GitError> {
    let mut command = Command::new("git");
    command.stderr(Stdio::inherit());

    f(&mut command);

    // Execute command
    let output = command.output().map_err(|_| GitError::Execute)?;

    if !output.status.success() {
        return Err(GitError::Unknown(output.status.code()));
    }

    Ok(output)
}
