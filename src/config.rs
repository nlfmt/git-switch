use dirs::config_dir;
use std::{
    fs,
    process::{Command, ExitStatus},
};

fn get_config_dir() -> Option<std::path::PathBuf> {
    config_dir().map(|path| path.join("git-switch-branch"))
}

fn get_alias_file() -> std::io::Result<std::path::PathBuf> {
    let config_dir = get_config_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")
    })?;
    fs::create_dir_all(config_dir.clone())?;

    Ok(config_dir.join("alias"))
}

pub fn get_current_alias() -> std::io::Result<Option<String>> {
    let alias_file = get_alias_file()?;
    if !alias_file.exists() {
        Ok(None)
    } else {
        Ok(Some(fs::read_to_string(alias_file)?))
    }
}

pub fn remove_alias(alias: Option<&str>) -> std::io::Result<()> {
    let alias_file = get_alias_file()?;
    if alias_file.exists() {
        fs::remove_file(alias_file)?;
    }

    if let Some(alias) = alias {
        remove_git_alias(alias)?;
    }

    Ok(())
}

fn remove_git_alias(alias: &str) -> std::io::Result<ExitStatus> {
    Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("--unset")
        .arg(format!("alias.{}", alias))
        .status()
}

pub fn update_alias(alias: &str, previous: Option<&str>) -> std::io::Result<()> {
    if let Some(previous) = previous {
        remove_git_alias(previous)?;
    }

    Command::new("git")
        .arg("config")
        .arg("--global")
        .arg(format!("alias.{}", alias))
        .arg("!git-switch-branch")
        .status()?;

    let alias_file = get_alias_file()?;
    fs::write(alias_file, alias)?;

    Ok(())
}
