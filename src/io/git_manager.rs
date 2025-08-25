use chrono::prelude::*;
use std::{path, rc};

use crate::error::RucolaError;

/// Manages interaction with the `git2` library.
#[derive(Clone)]
pub struct GitManager {
    /// The git repository the vault is stored in.
    git_repo: rc::Rc<git2::Repository>,
}

impl GitManager {
    /// Checks if the given path is contained in a git repository, and if yes, creates an object managing that repository.
    pub fn new(vault_path: path::PathBuf) -> Option<Self> {
        git2::Repository::discover(vault_path)
            .map(|git_repo| Self {
                git_repo: rc::Rc::new(git_repo),
            })
            .ok()
    }

    /// Performs the equivalent of `git add .` on the repository.
    pub fn add_all(&self) -> Result<(), RucolaError> {
        let mut index = self.git_repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?; // save updated index
        Ok(())
    }

    /// Performs the equivalent of `git commit -m "Rucola commit at <UTC DateTime> on <Hostname>." on the repository.`
    pub fn commit(&self) -> Result<(), RucolaError> {
        // Take the index and write it to a tree.
        let mut index = self.git_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git_repo.find_tree(tree_oid)?;

        // Get the parent commit.
        let parent_commit = self
            .git_repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .and_then(|oid| self.git_repo.find_commit(oid).ok());

        // Read out user config to get Name + Email.
        let sig = self.git_repo.signature()?;

        // Create the message.
        let message = format!(
            "Rucola commit at {} on {}.",
            Utc::now(),
            gethostname::gethostname()
                .into_string()
                .unwrap_or("unknown host".to_owned())
        );

        // Create the commit.
        match parent_commit {
            Some(parent) => {
                self.git_repo
                    .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent])?
            }
            None => self
                .git_repo
                .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])?,
        };
        Ok(())
    }

    /// Calculates how many commits the current branch is ahead/behind compared to its origin.
    pub fn calculate_ahead_behind(&self) -> Option<(usize, usize)> {
        let head = self.git_repo.head().ok()?;
        let head_id = head.target()?;

        let branch = self
            .git_repo
            .find_branch(head.shorthand()?, git2::BranchType::Local)
            .ok()?;

        let upstream = branch.upstream().ok()?;

        let upstream_id = upstream.get().target()?;

        self.git_repo.graph_ahead_behind(head_id, upstream_id).ok()
    }

    /// Checks if there are any untracked or uncommited changes in the repository at the current time.
    pub fn changes(&self) -> (bool, bool) {
        let mut status_options = git2::StatusOptions::new();

        let statuses = self.git_repo.statuses(Some(
            status_options
                .include_untracked(true)
                .recurse_untracked_dirs(true)
                .include_ignored(false)
                .exclude_submodules(true),
        ));

        if statuses.is_err() {
            return (false, false);
        }

        let statuses = statuses.unwrap();

        (
            statuses.iter().any(|entry| {
                let status = entry.status();
                status.is_wt_modified()
                    || status.is_wt_deleted()
                    || status.is_wt_new()
                    || status.is_wt_typechange()
                    || status.is_wt_renamed()
            }),
            statuses.iter().any(|entry| {
                let status = entry.status();
                status.is_index_modified()
                    || status.is_index_deleted()
                    || status.is_index_new()
                    || status.is_index_typechange()
                    || status.is_index_renamed()
            }),
        )
    }
}
