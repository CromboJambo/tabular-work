// git-sheets publish: GitHub integration — create repo and push package

use crate::core::{GitSheetsError, Result};
use std::path::Path;
use std::process::Command;

/// Push a built publish package to a new GitHub repository.
/// Uses `gh` CLI for auth and repo creation when available.
pub fn push_to_github(
    package_dir: &Path,
    title: &str,
    description: Option<&str>,
    repo_name: &str,
) -> Result<String> {
    let owner = current_user()?;

    println!("Creating GitHub repository {}...", repo_name);

    // Check if `gh` CLI is available
    let has_gh = Command::new("which")
        .arg("gh")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let repo_url;
    if has_gh {
        // Use gh CLI to create repo and push
        repo_url = push_via_gh_cli(package_dir, &owner, repo_name, title, description)?;
    } else {
        // Fall back to direct git with credential helper
        repo_url = push_via_git_direct(package_dir, &owner, repo_name, title)?;
    }

    println!("✓ Published to {}", repo_url);
    Ok(repo_url)
}

/// Push using the `gh` CLI tool (preferred method)
fn push_via_gh_cli(
    package_dir: &Path,
    owner: &str,
    repo_name: &str,
    title: &str,
    description: Option<&str>,
) -> Result<String> {
    let full_repo = format!("{}/{}", owner, repo_name);

    // Check if repo already exists
    let check = Command::new("gh")
        .args(&["repo", "view", &full_repo, "--json", "name"])
        .output();

    let needs_create = match check {
        Ok(output) => !output.status.success(),
        Err(_) => true,
    };

    if needs_create {
        // Create new repo
        let mut cmd = Command::new("gh");
        cmd.args(&["repo", "create", &full_repo]);

        if let Some(desc) = description {
            cmd.args(&["--description", desc]);
        }

        let output = cmd.output().map_err(|e| {
            GitSheetsError::FileSystemError(format!("Failed to run gh repo create: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitSheetsError::FileSystemError(format!(
                "gh repo create failed: {}",
                stderr.trim()
            )));
        }
    }

    // Initialize git in the package directory
    Command::new("git")
        .args(&["init"])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to init git: {}", e)))?;

    // Add all files
    Command::new("git")
        .args(&["add", "."])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to add files: {}", e)))?;

    // Commit
    let commit_msg = format!("Initial workflow: {}", title);
    Command::new("git")
        .args(&["commit", "-m", &commit_msg])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to commit: {}", e)))?;

    // Push using gh (handles auth automatically)
    let output = Command::new("gh")
        .args(&["repo", "push", "--force"])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to push: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitSheetsError::FileSystemError(format!(
            "gh repo push failed: {}",
            stderr.trim()
        )));
    }

    Ok(format!("https://github.com/{}", full_repo))
}

/// Push using plain git with credential helper (fallback)
fn push_via_git_direct(
    package_dir: &Path,
    owner: &str,
    repo_name: &str,
    title: &str,
) -> Result<String> {
    // First check if user has a working git remote configured
    let origin_check = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .current_dir(package_dir)
        .output();

    let repo_url = format!("https://github.com/{}/{}.git", owner, repo_name);

    // Initialize git in the package directory
    Command::new("git")
        .args(&["init"])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to init git: {}", e)))?;

    // Add all files
    Command::new("git")
        .args(&["add", "."])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to add files: {}", e)))?;

    // Commit
    let commit_msg = format!("Initial workflow: {}", title);
    Command::new("git")
        .args(&["commit", "-m", &commit_msg])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to commit: {}", e)))?;

    // Add remote and push
    Command::new("git")
        .args(&["remote", "add", "origin", &repo_url])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to add remote: {}", e)))?;

    let output = Command::new("git")
        .args(&["push", "-u", "origin", "main"])
        .current_dir(package_dir)
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to push: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitSheetsError::FileSystemError(format!(
            "git push failed: {}",
            stderr.trim()
        )));
    }

    Ok(repo_url)
}

/// Determine current GitHub user from `gh` CLI or git config
fn current_user() -> Result<String> {
    // Try gh CLI first
    let output = Command::new("gh")
        .args(&["api", "user", "--jq", ".login"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !user.is_empty() {
                return Ok(user);
            }
        }
    }

    // Fall back to git config
    let output = Command::new("git")
        .args(&["config", "--global", "user.name"])
        .output()
        .map_err(|e| GitSheetsError::FileSystemError(format!("Failed to get git user: {}", e)))?;

    if !output.status.success() {
        return Err(GitSheetsError::FileSystemError(
            "Could not determine GitHub username. Set up gh CLI or git config user.name"
                .to_string(),
        ));
    }

    let user = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(user)
}
