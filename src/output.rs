use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};

pub fn clear() {
    let _ = execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    let _ = std::io::stdout().flush().unwrap();
}

pub async fn print_top(text: &str) {
    let pos = cursor::position().unwrap();
    let _ = execute!(io::stdout(), cursor::MoveTo(0, 0),).unwrap();
    print!("{}", text);
    io::stdout().flush().unwrap();
    let _ = execute!(io::stdout(), cursor::MoveTo(pos.0, pos.1),);
}
