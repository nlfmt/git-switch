use std::{
    env,
    io::{stdout, Write},
};

pub use commands::*;
use menu::{run_menu, Menu};
use repo::{checkout, current_branch, get_branches, root_repo_path, Branch};

mod commands;
mod config;
mod menu;
mod repo;
mod util;

pub fn help(version: &str) {
    println!("git-switch-branch v{}", version);
    println!("Usage: git-switch-branch [options]");
    println!("  help           Show this help message");
    println!("  version        Show the version number");
    println!("");
    println!("  r, remote      Show only remote branches");
    println!("  a, all         Show all branches (local and remote)");
    println!("");
    println!("  alias add      Add a git alias for this command");
    println!("  alias remove   Remove the git alias (if set)");
    println!("  alias          Show the current alias");
}

pub fn list_alias() {
    let alias = config::get_current_alias().unwrap_or_else(|_| fatal!("Could not read alias file"));

    match alias {
        Some(alias) => println!("{}", alias),
        None => println!("No alias set"),
    }
}

pub fn add_alias() {
    let alias = config::get_current_alias().unwrap_or_else(|_| fatal!("Could not read alias file"));

    if alias.is_some() {
        println!("Current alias: {}", alias.as_ref().unwrap());
    }

    let mut input = String::new();

    let new_alias = loop {
        print!("Enter new alias: ");
        stdout().flush().unwrap();

        let stdin = std::io::stdin();
        stdin.read_line(&mut input).unwrap();
        let alias = input.trim();

        if alias.is_empty() {
            println!("Alias cannot be empty");
            input.clear();
            continue;
        }

        break alias;
    };

    config::update_alias(new_alias, alias.as_deref())
        .unwrap_or_else(|_| fatal!("Could not update alias"));

    println!("Alias updated");
}

pub fn remove_alias() {
    let alias = config::get_current_alias().unwrap_or_else(|_| fatal!("Could not read alias file"));
    if alias.is_none() {
        println!("No alias set");
        return;
    }

    config::remove_alias(alias.as_deref()).unwrap_or_else(|_| fatal!("Could not remove alias"));
    println!("Alias removed");
}

pub fn switch_branch(local: bool, remote: bool) {
    let repo_path = env::current_dir()
        .map(|mut path| root_repo_path(&mut path))
        .expect("Can't access current directory")
        .unwrap_or_else(|| fatal!("Could not find git repository"));

    let branches = get_branches(&repo_path, local, remote)
        .unwrap_or_else(|_| fatal!("Could not read branches"));

    let current_branch = current_branch(&repo_path)
        .map(|res| branches.iter().position(|branch| branch.to_string() == res))
        .unwrap_or_default();

    if branches.len() == 0 {
        println!("No branches found");
        return;
    }

    let menu = Menu {
        items: &branches,
        current: current_branch,
        ..Default::default()
    };

    let selected = run_menu(&menu).map(|i| &branches[i]).unwrap_or_else(|err| {
        fatal!("Could not run menu: {}", err);
    });

    let branch = match &selected {
        Branch::Remote(_, branch) => {
            let opts = vec![
                format!("Create local branch '{}'", branch),
                "Checkout remote branch (detached HEAD)".to_owned(),
            ];
            let menu = Menu {
                items: &opts,
                ..Default::default()
            };

            let opt = run_menu(&menu).unwrap_or_else(|err| fatal!("Could not run menu: {}", err));

            match opt {
                0 => branch,
                _ => &selected.to_string(),
            }
        }
        Branch::Local(branch) => branch,
    };

    checkout(branch).unwrap_or_else(|_| fatal!("Could not checkout branch"));
}
