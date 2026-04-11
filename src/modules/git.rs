use std::path::Path;

use crate::util::process;

use super::InfoLine;

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub available: bool,
    pub is_repository: bool,
    pub branch: Option<String>,
    pub head_short: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum GitView {
    Branch,
    #[default]
    BranchOrCommit,
}

impl GitInfo {
    pub fn inspect(dir: &Path) -> Self {
        let inside_work_tree =
            match process::run_in_dir("git", &["rev-parse", "--is-inside-work-tree"], dir) {
                Ok(output) => output.success && output.stdout == "true",
                Err(_) => {
                    return Self {
                        available: false,
                        is_repository: false,
                        branch: None,
                        head_short: None,
                    };
                }
            };

        if !inside_work_tree {
            return Self {
                available: true,
                is_repository: false,
                branch: None,
                head_short: None,
            };
        }

        let branch = process::run_in_dir("git", &["branch", "--show-current"], dir)
            .ok()
            .map(|output| output.stdout)
            .filter(|branch| !branch.is_empty());

        let head_short = process::run_in_dir("git", &["rev-parse", "--short", "HEAD"], dir)
            .ok()
            .map(|output| output.stdout)
            .filter(|head| !head.is_empty());

        Self {
            available: true,
            is_repository: true,
            branch,
            head_short,
        }
    }

    pub fn render(&self, view: GitView) -> Option<InfoLine> {
        if !self.available {
            return None;
        }

        if !self.is_repository {
            return Some(InfoLine::new("Git", "not a repository"));
        }

        let value = match view {
            GitView::Branch => self.branch.clone().or_else(|| self.head_short.clone())?,
            GitView::BranchOrCommit => match (&self.branch, &self.head_short) {
                (Some(branch), _) => branch.clone(),
                (None, Some(head_short)) => format!("detached @ {}", head_short),
                (None, None) => return None,
            },
        };

        Some(InfoLine::new("Git", value))
    }
}
