pub enum Command {
    Help,
    Version,
    Alias,
    Remote,
    All,
    Invalid,
}

use Command::*;

impl Command {
    pub fn from(arg: &str) -> Command {
        match arg {
            "help" | "--help" | "-h" => Help,
            "version" | "--version" | "-v" => Version,
            "alias" => Alias,
            "remote" | "r" => Remote,
            "all" | "a" => All,
            _ => Invalid,
        }
    }
}
