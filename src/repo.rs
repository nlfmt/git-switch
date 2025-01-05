use std::path::{Path, PathBuf};

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

pub fn get_branches(repo_path: &impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let path = repo_path.as_ref().join(".git/refs/heads");

    get_files(path, None)
}

pub fn get_remote_branches(repo_path: &impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let path = repo_path.as_ref().join(".git/refs/remotes");

    get_files(path, None)
}

fn get_files(directory: impl AsRef<Path>, parent: Option<&Path>) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in directory.as_ref().read_dir()? {
        let path = entry?.path();

        if path.is_dir() {
            files.append(&mut get_files(&path, Some(&path))?);
        } else {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if let Some(parent) = parent {
                let parent = parent.file_name().unwrap().to_str().unwrap();
                files.push(format!("{}/{}", parent, file_name));
            } else {
                files.push(file_name.to_string());
            }
        }
    }
    Ok(files)
}
