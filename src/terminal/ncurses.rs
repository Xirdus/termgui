extern crate ncurses;
use std::cmp::{min, max};
use std::mem::uninitialized;
use std::ptr::null_mut;
use color::ColorPair;
use color::Color::*;
use terminal::Terminal;

pub struct SystemTerminal {
    default_color: ColorPair,
}

/// Initializes system terminal. **WARNING: Must NOT be called more than once in runtime.**
pub fn init_terminal() -> Result<SystemTerminal,()> {
    if ncurses::initscr() == null_mut()
    || ncurses::noecho() != ncurses::OK
    || ncurses::start_color() != ncurses::OK {
        return Err(())
    }
        
    for i in 0..8 {
        for j in 0..8 {
            if ncurses::init_pair(i * 8 + j + 1, i, j) != ncurses::OK {
                return Err(())
            }
        }
    }
    
    Ok(SystemTerminal { default_color: ColorPair { fg: Some(DarkWhite), bg: Some(DarkBlack) }})
}

impl Terminal for SystemTerminal {
    fn size(&self) -> (u16, u16) {
        let mut x = unsafe { uninitialized() };
        let mut y = unsafe { uninitialized() };
        ncurses::getmaxyx(ncurses::stdscr, &mut y, &mut x);
        (x as u16, y as u16)
    }

    fn resize(&mut self, _: u16, _: u16) -> Result<(),()> {
        Err(()) // unsupported
    }

    fn get_cursor_pos(&self) -> (u16, u16) {
        let mut x = unsafe { uninitialized() };
        let mut y = unsafe { uninitialized() };
        ncurses::getyx(ncurses::stdscr, &mut y, &mut x);
        (x as u16, y as u16)
    }

    fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),()> {
        if ncurses::mv(y as i32, x as i32) == ncurses::OK {
            Ok(())
        } else {
            Err(())
        }
    }

    fn get_default_color(&self) -> ColorPair {
        self.default_color
    }

    fn set_default_color(&mut self, color: ColorPair) {
        self.default_color = color.compose(self.default_color);
    }
    
    #[allow(unused_must_use)]
    fn print_at<T>(&mut self, x: i16, y: i16, line: T) where T: Iterator<Item=(char, ColorPair)> {
        let (max_x, max_y) = self.size();
        if y < 0 || y as u16 > max_y {
            return;
        }
        let (old_x, old_y) = self.get_cursor_pos();
        self.set_cursor_pos(min(max(0, x) as u16, max_x - 1), y as u16);
        
        let s = max(0 - x, 0) as usize;
        for (ch, co) in line.skip(s).take_while(|&(c, _)| c != '\n') {
            set_color(co.compose(self.default_color));
            ncurses::addch(ch as u64);
        }
        self.set_cursor_pos(old_x, old_y);
    }
    
    fn present(&mut self) {
        ncurses::refresh();
    }
}

impl Drop for SystemTerminal {
    fn drop(&mut self) {
        ncurses::endwin();
    }
}

fn set_color(color: ColorPair) {
    ncurses::attrset(match color.fg.unwrap() {
        LightBlack| LightRed | LightGreen | LightYellow | LightBlue | LightMagenta | LightCyan
        | LightWhite => ncurses::A_BOLD(),
        _ => 0,
    } | ncurses::COLOR_PAIR(match color.fg.unwrap() {
        DarkBlack | LightBlack => 0,
        DarkRed | LightRed => 1,
        DarkGreen | LightGreen => 2,
        DarkYellow | LightYellow => 3,
        DarkBlue | LightBlue => 4,
        DarkMagenta | LightMagenta => 5,
        DarkCyan | LightCyan => 6,
        DarkWhite | LightWhite => 7,
    } * 8 + match color.bg.unwrap() {
        DarkBlack | LightBlack => 0,
        DarkRed | LightRed => 1,
        DarkGreen | LightGreen => 2,
        DarkYellow | LightYellow => 3,
        DarkBlue | LightBlue => 4,
        DarkMagenta | LightMagenta => 5,
        DarkCyan | LightCyan => 6,
        DarkWhite | LightWhite => 7,
    } + 1));
}
