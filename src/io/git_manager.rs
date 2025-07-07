use std::{path, rc};

/// Manages interaction with the `git2` library.
#[derive(Clone)]
pub struct GitManager {
    /// The git repository the vault is stored in.
    git_repo: rc::Rc<git2::Repository>,
}

impl GitManager {
    // Checks if the given path is contained in a git repository, and if yes, creates an object managing that repository.
    pub fn new(vault_path: path::PathBuf) -> Option<Self> {
        git2::Repository::discover(vault_path)
            .map(|git_repo| Self {
                git_repo: rc::Rc::new(git_repo),
            })
            .ok()
    }

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
}
