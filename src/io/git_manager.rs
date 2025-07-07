use std::{path, rc};

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

    /// Calculates how many commits the current branch is ahead/behind compared to its origin.
    /// TODO: remove unwraps
    pub fn calculate_ahead_behind(&self) -> (usize, usize) {
        let head = self.git_repo.head().unwrap();
        let head_id = head.target().unwrap();

        let branch = self
            .git_repo
            .find_branch(head.shorthand().unwrap(), git2::BranchType::Local)
            .unwrap();

        let upstream = branch.upstream().unwrap();

        let upstream_id = upstream.get().target().unwrap();

        self.git_repo
            .graph_ahead_behind(head_id, upstream_id)
            .unwrap()
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
