use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    process::ExitStatus,
};

pub fn root_repo_path(path: &Path) -> Option<PathBuf> {
    let mut path = path.to_path_buf();

    loop {
        if path.join(".git").exists() {
            return Some(path);
        }

        if !path.pop() {
            return None;
        }
    }
}

pub fn current_branch(repo_path: &impl AsRef<Path>) -> std::io::Result<String> {
    let path = repo_path.as_ref().join(".git/HEAD");

    std::fs::read_to_string(path)?
        .strip_prefix("ref: refs/heads/")
        .map(|s| s.trim().to_string())
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not parse HEAD file")
        })
}

pub enum Branch {
    Local(String),
    Remote(String, String),
}
impl Display for Branch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Branch::Local(name) => write!(f, "{}", name),
            Branch::Remote(remote, name) => write!(f, "{}/{}", remote, name),
        }
    }
}

pub fn get_branches(
    repo_path: &impl AsRef<Path>,
    local: bool,
    remote: bool,
) -> std::io::Result<Vec<Branch>> {
    let mut branches: Vec<Branch> = Vec::new();

    if local {
        let path = repo_path.as_ref().join(".git/refs/heads");

        branches.extend(
            get_files(path, None)?
                .into_iter()
                .map(|name| Branch::Local(name)),
        );
    }

    if remote {
        let path = repo_path.as_ref().join(".git/refs/remotes");

        branches.extend(get_files(path, None)?.into_iter().map(|name| {
            let (remote, branch) = name.split_once('/').unwrap();
            Branch::Remote(remote.to_string(), branch.to_string())
        }));
    }

    Ok(branches)
}

fn get_files(directory: impl AsRef<Path>, parent: Option<String>) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in directory.as_ref().read_dir()? {
        let path = entry?.path();

        if path.is_dir() {
            let parent = join_path(&parent, last_segment(&path));
            files.append(&mut get_files(&path, Some(parent))?);
        } else {
            let file_name = last_segment(&path);
            files.push(join_path(&parent, file_name));
        }
    }
    Ok(files)
}

fn join_path(parent: &Option<String>, child: String) -> String {
    match parent {
        Some(parent) => format!("{}/{}", parent, child),
        None => child,
    }
}

fn last_segment(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}

pub fn checkout(branch: &str) -> std::io::Result<ExitStatus> {
    std::process::Command::new("git")
        .arg("checkout")
        .arg(branch)
        .status()
}
