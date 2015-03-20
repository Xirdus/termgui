//! Interaction with terminal.

use color::ColorPair;
use std::iter::repeat;
use windows::Window;

/// Represents access to terminal.
pub trait Terminal {
    /// Returns number of rows and columns in the terminal.
    fn size(&self) -> (u16, u16);

    /// Resizes the terminal. Returns an error if it's not possible to resize or if the size of
    /// the terminal after resizing doesn't match the requested values.
    fn resize(&mut self, width: u16, height: u16) -> Result<(),()>;

    /// Returns current position of text cursor.
    fn get_cursor_pos(&self) -> (u16, u16);

    /// Repositions the text cursor. Returns an error if the cursor cannot be placed at
    /// the requested position.
    fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),()>;

    /// Returns current default colors of the terminal.
    fn get_default_color(&self) -> ColorPair;

    /// Sets new default colors of the terminal. If either `fg` or `bg` is being set to None,
    /// the old color will be used for either foreground or background respectively.
    fn set_default_color(&mut self, color: ColorPair);

    /// Writes text string to the terminal at current position using default color. The cursor is
    /// moved after the text written.
    fn write(&mut self, s: &str) {
        let color = self.get_default_color();
        self.write_in_color(s, color);
    }

    /// Writes text string to the terminal at current position using given color. The cursor is
    /// moved after the text written.
    fn write_in_color(&mut self, s: &str, color: ColorPair) {
        self.write_colored(s, repeat(color));
    }

    /// Writes text string to the terminal at current position, with colors of consecutive
    /// characters taken from the iterator. If there are less colors in the iterator than letters
    /// in the text, the output will be truncated. The cursor is moved after the text written.
    #[allow(unused_must_use)]
    fn write_colored<T>(&mut self, s: &str, colors: T) where T: Iterator<Item=ColorPair> {
        let (max_x, max_y) = self.size();
        let (mut x, mut y) = self.get_cursor_pos();
        let mut cur_x = x;
        let mut cur_y = y;
        let mut i = s.chars().zip(colors).fuse().peekable();
        while y < max_y {
            if let Some(_) = i.peek() {
                cur_x = x;
                cur_y = y;
                self.print_at(x as i16, y as i16, i.by_ref().take((max_x - x) as usize)
                                                   .take_while(|&(c, _)| c != '\n')
                                                   .inspect(|&_| cur_x += 1));
                x = 0;
                y += 1;
            } else {
                break;
            }
        }
        self.set_cursor_pos(cur_x, cur_y);
        self.present();
    }

    /// Draws window in top left corner of terminal.
    fn draw<T: ?Sized>(&mut self, window: &T) where T: Window {
        window.draw(self, 0, 0);
        self.present();
    }

    /// Prints characters in single line starting at given coordinates. Mainly used by
    /// Window::draw() implementations. Lines are truncated at newline character or when the line
    /// exceeds terminal length. The result isn't shown until present() is called.
    fn print_at<T>(&mut self, x: i16, y: i16, line: T) where T: Iterator<Item=(char, ColorPair)>;

    /// Updates terminal contents after print_at() calls.
    fn present(&mut self);
}

#[cfg(windows)]
mod winapi;
#[cfg(windows)]
pub use self::winapi::SystemTerminal;
#[cfg(windows)]
pub use self::winapi::init_terminal;

#[cfg(unix)]
mod ncurses;
#[cfg(unix)]
pub use self::ncurses::SystemTerminal;
#[cfg(unix)]
pub use self::ncurses::init_terminal;
