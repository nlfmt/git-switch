use git_switch_branch::{add_alias, fatal, help, list_alias, remove_alias, switch_branch, Command};
use std::{env, process::exit};

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    match env::args().nth(1) {
        None => switch_branch(true, false),
        Some(arg) => match Command::from(&arg) {
            Command::Help => {
                help(version);
            }
            Command::Version => {
                println!("{}", version);
            }
            Command::Alias => match env::args().nth(2) {
                None => list_alias(),
                Some(v) => match v.as_str() {
                    "add" => add_alias(),
                    "remove" => remove_alias(),
                    _ => fatal!("Unknown option '{}'", v),
                },
            },
            Command::Remote => {
                switch_branch(false, true);
            }
            Command::All => {
                switch_branch(true, true);
            }
            Command::Invalid => {
                eprintln!("Unknown argument: {}", arg);
                help(version);
                exit(1);
            }
        },
    }
}
