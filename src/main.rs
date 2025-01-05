use std::{env, process::exit};
use git_switch_branch::{add_alias, help, list_alias, remove_alias, switch_branch};

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let mut remote = false;
    let mut local = true;
    
    if let Some(arg) = env::args().nth(1) {
        if arg == "help" || arg == "-h" || arg == "--help" {
            help(version);
            exit(0);
        }
        else if arg == "version" || arg == "-v" || arg == "--version" {
            println!("{}", version);
            exit(0);
        }
        else if arg == "add-alias" {
            add_alias();
            exit(0);
        }
        else if arg == "remove-alias" {
            remove_alias();
            exit(0);
        }
        else if arg == "alias" {
            list_alias();
            exit(0);
        }
        else if arg == "remote" || arg == "r" {
            remote = true;
            local = false;
        }
        else if arg == "all" || arg == "a" {
            remote = true;
        }
        else {
            eprintln!("Unknown argument: {}", arg);
            help(version);
            exit(1);
        }
    }
    
    switch_branch(local, remote);
}

