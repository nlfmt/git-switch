use std::{
    env,
    io::{stdout, Write},
    path::Path,
    process::{exit, Command},
};

use crossterm::{
    cursor,
    event::{
        self, Event,
        KeyCode::{Char, Down, Enter, Esc, Up},
        KeyEvent, KeyEventKind, KeyModifiers as Mod,
    },
    execute, queue,
    style::{self, ContentStyle, StyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

#[derive(Debug)]
struct Menu {
    items: Vec<String>,
    selected: usize,
    current: Option<usize>,
}

impl Menu {
    fn new(items: Vec<String>) -> Self {
        if items.len() == 0 {
            panic!("Menu must have at least one item");
        }
        Self {
            items,
            selected: 0,
            current: None,
        }
    }

    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }

    fn prev(&mut self) {
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    fn selected(&self) -> &str {
        &self.items[self.selected]
    }

    fn set_current(&mut self, current: usize) {
        self.current = Some(current);
    }

    fn print(&self) {
        for (i, item) in self.items.iter().enumerate() {
            if i == self.selected {
                queue!(
                    stdout(),
                    style::Print("> "),
                    style::PrintStyledContent(StyledContent::new(
                        ContentStyle::new().underlined().green(),
                        format!("{}", item)
                    ))
                )
                .unwrap();
            } else {
                queue!(stdout(), style::Print(format!("  {}", item))).unwrap();
            }
            if let Some(current) = self.current {
                if i == current {
                    queue!(
                        stdout(),
                        style::PrintStyledContent(StyledContent::new(
                            ContentStyle::new().yellow(),
                            " (current)"
                        ))
                    )
                    .unwrap();
                }
            }
            queue!(stdout(), style::Print("\n")).unwrap();
        }
    }

    fn unprint(&self) {
        queue!(
            stdout(),
            cursor::MoveUp(self.items.len() as u16),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )
        .unwrap();
    }
}

fn begin() -> std::io::Result<()> {
    execute!(stdout(), cursor::Hide).unwrap();
    enable_raw_mode()
}

fn end() -> std::io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), cursor::Show)?;
    stdout().flush()
}

fn run_menu(menu: &mut Menu) -> std::io::Result<()> {
    menu.print();

    begin()?;
    let _ = event::read()?;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = event::read()?
            {
                if kind != KeyEventKind::Release {
                    match (code, modifiers) {
                        (Char('j'), Mod::NONE) | (Char('s'), Mod::NONE) | (Down, Mod::NONE) => {
                            menu.next()
                        }
                        (Char('k'), Mod::NONE) | (Char('w'), Mod::NONE) | (Up, Mod::NONE) => {
                            menu.prev()
                        }

                        (Char('c'), Mod::CONTROL) | (Char('q'), Mod::NONE) | (Esc, Mod::NONE) => {
                            menu.unprint();
                            end()?;
                            exit(0);
                        }

                        (Enter, Mod::NONE) | (Char(' '), _) | (Char('l'), Mod::NONE) => {
                            menu.unprint();
                            end()?;
                            return Ok(());
                        }

                        _ => {}
                    }

                    menu.unprint();
                    menu.print();
                    stdout().flush().unwrap();
                }
            }
        }
    }
}

fn current_branch(repo_path: &impl AsRef<Path>) -> Option<String> {
    let mut path = repo_path.as_ref().to_owned();
    path.push(".git/HEAD");

    if !path.exists() {
        return None;
    }

    let head = std::fs::read_to_string(path).unwrap();
    let head = head.trim_start_matches("ref: refs/heads/").trim();

    Some(head.to_string())
}

fn get_branches(repo_path: &impl AsRef<Path>) -> Option<Vec<String>> {
    let mut path = repo_path.as_ref().to_owned();
    path.push(".git/refs/heads");

    if !path.exists() {
        return None;
    }

    Some(get_files(path, None).unwrap())
}

fn get_files(
    directory: impl AsRef<Path>,
    parent: Option<&Path>,
) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    for entry in directory.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();

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

fn main() {
    let repo_path = env::current_dir().expect("Can't access current directory");

    let branches = get_branches(&repo_path).unwrap_or_else(|| {
        println!("Not a git repository");
        std::process::exit(1);
    });

    let current_branch = current_branch(&repo_path)
        .map(|b| branches.iter().position(|i| i == &b))
        .flatten();

    if branches.len() == 0 {
        println!("No branches found");
        return;
    }

    let mut menu = Menu::new(branches);
    if let Some(current) = current_branch {
        menu.set_current(current);
    }

    run_menu(&mut menu).unwrap();

    println!("checking out '{}'", menu.selected());

    let _ = Command::new("git")
        .arg("checkout")
        .arg(menu.selected())
        .status()
        .expect("Failed to execute git");
}
