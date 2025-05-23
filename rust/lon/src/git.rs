use std::{
    fmt,
    path::Path,
    process::{Command, Output},
};

use anyhow::{bail, Context, Result};
use tempfile::TempDir;

/// A git revision (aka commit).
#[derive(PartialEq, Clone)]
pub struct Revision(String);

impl Revision {
    pub fn new(s: &str) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Output of `git ls-remote`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RemoteInfo {
    pub revision: String,
    pub reference: String,
}

/// Find the newest revision for a branch of a git repository.
pub fn find_newest_revision(url: &str, branch: &str) -> Result<Revision> {
    find_newest_revision_for_ref(url, &format!("refs/heads/{branch}")).with_context(|| {
        format!(
            "Failed to find newest revision for {url} ({branch}).\nAre you sure the repo exists and contains the branch {branch}?"
        )
    })
}

/// Find the newest revision for a reference of a git repository.
fn find_newest_revision_for_ref(url: &str, reference: &str) -> Result<Revision> {
    let mut references =
        ls_remote(&["--refs", url, reference]).with_context(|| format!("Failed to reach {url}"))?;

    if references.is_empty() {
        bail!("The repository {url} doesn't contain the reference {reference}")
    }

    if references.len() > 1 {
        bail!("The reference {reference} is ambiguous and points to multiple revisions")
    }

    Ok(Revision(references.remove(0).revision))
}

/// Call `git ls-remote` with the provided args.
fn ls_remote(args: &[&str]) -> Result<Vec<RemoteInfo>> {
    let output = Command::new("git")
        .arg("ls-remote")
        .args(args)
        .output()
        .context("Failed to execute git ls-remote. Most likely it's not on PATH")?;
    if !output.status.success() {
        let status_code = output
            .status
            .code()
            .map_or_else(|| "None".into(), |code| code.to_string());
        let stderr_output = String::from_utf8_lossy(&output.stderr)
            .lines()
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");
        anyhow::bail!("git ls-remote failed with exit code {status_code}:\n{stderr_output}",);
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (revision, reference) = line.split_once('\t').ok_or_else(|| {
                anyhow::format_err!("git ls-remote output line contains no '\\t'")
            })?;
            if reference.contains('\t') {
                bail!("git ls-remote output line contains more than one '\\t'")
            }
            Ok(RemoteInfo {
                revision: revision.into(),
                reference: reference.into(),
            })
        })
        .collect::<Result<Vec<RemoteInfo>>>()
}

/// Obtain the lastModified information
pub fn get_last_modified(url: &str, rev: &str) -> Result<u64> {
    let tmp_dir = TempDir::new()?;
    let mut output: Output;

    // Init a new git directory
    output = Command::new("git")
        .arg("--git-dir")
        .arg(tmp_dir.path())
        .arg("init")
        .output()
        .context("Failed to execute git init. Most likely it's not on PATH")?;

    if !output.status.success() {
        bail!(
            "Failed to initialize a fresh git repository\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    }

    // Add the repository as a remote
    output = Command::new("git")
        .arg("--git-dir")
        .arg(tmp_dir.path())
        .args(["remote", "add", "origin", url])
        .output()
        .context("Failed to execute git remote add.")?;

    if !output.status.success() {
        bail!(
            "Failed to add the remote {}\n{}",
            url,
            String::from_utf8_lossy(&output.stderr)
        )
    }

    // Fetch the locked revision
    output = Command::new("git")
        .arg("--git-dir")
        .arg(tmp_dir.path())
        .args([
            "fetch",
            "--depth=1",
            "--no-show-forced-updates",
            "origin",
            rev,
        ])
        .output()
        .context("Failed to execute git fetch.")?;

    if !output.status.success() {
        bail!(
            "Failed to fetch the revision {}\n{}",
            rev,
            String::from_utf8_lossy(&output.stderr)
        )
    }

    // Get the lastModified value
    output = Command::new("git")
        .arg("--git-dir")
        .arg(tmp_dir.path())
        .args(["log", "-1", "--format=%ct", "--no-show-signature", rev])
        .output()
        .context("Failed to execute git log.")?;

    if !output.status.success() {
        bail!(
            "Failed to log the revision {}\n{}",
            rev,
            String::from_utf8_lossy(&output.stderr)
        )
    }

    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .parse::<u64>()
        .context("Failed to parse last modified timestamp.")
}

pub fn add(directory: impl AsRef<Path>, args: &[&Path]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory.as_ref())
        .arg("add")
        .args(args)
        .output()
        .context("Failed to execute git add. Most likely it's not on PATH")?;

    if !output.status.success() {
        bail!(
            "Failed to add files to git statging\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub fn commit(directory: impl AsRef<Path>, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory.as_ref())
        .arg("commit")
        .arg("--message")
        .arg(message)
        .output()
        .context("Failed to execute git commit. Most likely it's not on PATH")?;

    if !output.status.success() {
        bail!(
            "Failed to commit files\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
