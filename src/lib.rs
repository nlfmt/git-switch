use std::{env, io::{stdout, Write}, process::Command};

use menu::{run_menu, Menu};
use repo::{current_branch, get_branches, get_remote_branches, root_repo_path};

mod menu;
mod config;
mod util;
mod repo;


pub fn fatal(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}

pub fn help(version: &str) {
    println!("git-switch-branch v{}", version);
    println!("Usage: git-switch-branch [options]");
    println!("  help           Show this help message");
    println!("  version        Show the version number");
    println!("");
    println!("  r, remote      Show only remote branches");
    println!("  a, all         Show all branches (local and remote)");
    println!("");
    println!("  add-alias      Add a git alias for this command");
    println!("  remove-alias   Remove the git alias (if set)");
    println!("  alias          Show the current alias");
}

pub fn list_alias() {
    let alias = config::get_current_alias()
        .unwrap_or_else(|_| fatal("Could not read alias file"));
    
    match alias {
        Some(alias) => println!("{}", alias),
        None => println!("No alias set"),
    }
}

pub fn add_alias() {
    let alias = config::get_current_alias()
        .unwrap_or_else(|_| fatal("Could not read alias file"));
    
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
        .unwrap_or_else(|_| fatal("Could not update alias"));
    
    println!("Alias updated");
}

pub fn remove_alias() {
    let alias = config::get_current_alias()
        .unwrap_or_else(|_| fatal("Could not read alias file"));
    if alias.is_none() {
        println!("No alias set");
        return;
    }

    config::remove_alias(alias.as_deref()).unwrap_or_else(|_| fatal("Could not remove alias"));
    println!("Alias removed");
}

pub fn switch_branch(local: bool, remote: bool) { 
    let repo_path = env::current_dir()
        .map(|mut path| root_repo_path(&mut path))
        .expect("Can't access current directory")
        .unwrap_or_else(|| fatal("Could not find git repository"));

    let mut branches: Vec<String> = Vec::new();

    if local {
        branches.extend(get_branches(&repo_path)
            .unwrap_or_else(|_| fatal("Could not read branches from .git directory")));
    }
    if remote {
        branches.extend(get_remote_branches(&repo_path)
            .unwrap_or_else(|_| fatal("Could not read remote branches from .git directory")));
    }

    let current_branch = current_branch(&repo_path).map(|res| {
        branches.iter().position(|branch| branch == &res)
    }).unwrap_or_default();
    // let current_branch = match current_branch(&repo_path) {
    //     Err(_) => fatal("Could not read current branch from .git directory"),
    //     Ok(res) => branches
    //         .iter()
    //         .position(|branch| branch == &res)
    // };

    if branches.len() == 0 {
        println!("No branches found");
        return;
    }

    let menu = Menu {
        items: branches.clone(),
        current: current_branch,
        ..Default::default()
    };

    let selected = run_menu(&menu)
        .map(|i| &branches[i])
        .unwrap_or_else(|err| {
            fatal(&format!("Could not run menu: {}", err));
        });

    Command::new("git")
        .arg("checkout")
        .arg(selected)
        .status()
        .unwrap_or_else(|_| fatal("Failed to execute git"));
}