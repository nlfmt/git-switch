use std::{fmt::Display, io::stdout, process::exit, time::Duration};

use crossterm::{
    cursor::{self, MoveTo},
    event::{
        self,
        KeyCode::{Char, Down, Enter, Esc, Up},
        KeyEvent, KeyModifiers as Mod,
    },
    execute, queue,
    style::{ContentStyle, Print, PrintStyledContent, StyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::util::{poll_keypress, CircularCounter};

pub struct Menu<'a, T: Display> {
    pub items: &'a [T],
    pub current: Option<usize>,
    pub current_label: String,
    pub current_label_style: ContentStyle,
    pub selected_style: ContentStyle,
}

impl<'a, T: Display> Default for Menu<'a, T> {
    fn default() -> Self {
        Self {
            items: &[],
            current: None,
            current_label: "(current)".to_string(),
            current_label_style: ContentStyle::new().yellow(),
            selected_style: ContentStyle::new().underlined().green(),
        }
    }
}

fn print_menu<T: Display>(menu: &Menu<T>, selected: usize) -> std::io::Result<()> {
    for i in 0..menu.items.len() {
        print_menu_item(menu, i, i == selected)?;

        if menu.current.is_some_and(|current| i == current) {
            queue!(
                stdout(),
                PrintStyledContent(StyledContent::new(
                    menu.current_label_style,
                    format!(" {}", menu.current_label)
                ))
            )?;
        }

        queue!(stdout(), Print("\n"))?;
    }
    Ok(())
}

fn print_menu_item<T: Display>(
    menu: &Menu<T>,
    index: usize,
    selected: bool,
) -> std::io::Result<()> {
    let item = &menu.items[index];
    if selected {
        queue![
            stdout(),
            Print("> "),
            PrintStyledContent(StyledContent::new(menu.selected_style, item)),
        ]?;
    } else {
        queue!(stdout(), Print(format!("  {}", item)))?;
    }
    Ok(())
}

fn unprint(line_count: usize) -> std::io::Result<()> {
    queue!(
        stdout(),
        cursor::MoveUp(line_count as u16),
        terminal::Clear(terminal::ClearType::FromCursorDown),
    )
}

#[derive(Debug)]
enum MenuAction {
    Next,
    Prev,
    Select,
    Exit,
    None,
}
impl MenuAction {
    fn from_key_event(event: KeyEvent) -> Self {
        match (event.code, event.modifiers) {
            (Char('j'), Mod::NONE) | (Char('s'), Mod::NONE) | (Down, Mod::NONE) => MenuAction::Next,
            (Char('k'), Mod::NONE) | (Char('w'), Mod::NONE) | (Up, Mod::NONE) => MenuAction::Prev,
            (Char('c'), Mod::CONTROL) | (Char('q'), Mod::NONE) | (Esc, Mod::NONE) => {
                MenuAction::Exit
            }
            (Enter, Mod::NONE) | (Char(' '), _) | (Char('l'), Mod::NONE) => MenuAction::Select,
            _ => MenuAction::None,
        }
    }
}

pub fn run_menu<T: Display>(menu: &Menu<T>) -> std::io::Result<usize> {
    let mut stdout = stdout();
    let mut counter = CircularCounter {
        size: menu.items.len(),
        selected: 0,
    };

    print_menu(menu, counter.selected)?;
    enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    // clear out previous events
    if event::poll(std::time::Duration::ZERO)? {
        let _ = event::read()?;
    }

    loop {
        if let Some(key_event) = poll_keypress(Duration::from_millis(100))? {
            match MenuAction::from_key_event(key_event) {
                a @ MenuAction::Next | a @ MenuAction::Prev => {
                    let start = cursor::position()?;
                    let y0 = start.1 - menu.items.len() as u16;
                    let prev = counter.selected;
                    let next = match a {
                        MenuAction::Next => counter.next(),
                        MenuAction::Prev => counter.prev(),
                        _ => unreachable!(),
                    };

                    queue!(stdout, MoveTo(0, y0 + prev as u16))?;
                    print_menu_item(&menu, prev, false)?;
                    queue!(stdout, MoveTo(0, y0 + next as u16))?;
                    print_menu_item(&menu, next, true)?;
                    execute!(stdout, MoveTo(start.0, start.1))?;
                }
                a @ MenuAction::Select | a @ MenuAction::Exit => {
                    unprint(menu.items.len())?;
                    disable_raw_mode()?;
                    execute!(stdout, cursor::Show)?;
                    match a {
                        MenuAction::Select => return Ok(counter.selected),
                        MenuAction::Exit => exit(0),
                        _ => unreachable!(),
                    }
                }
                _ => {}
            };
        }
    }
}
