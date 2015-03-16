//! Interaction with terminal.

use color::ColorPair;
use std::iter::repeat;

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
    /// the requested spot.
    fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),()>;
    /// Returns current default colors of the terminal. It's guaranteed to have both foreground and
    /// background colors filled.
    fn get_default_color(&self) -> ColorPair;
    /// Sets new default colors of the terminal. If either `fg` or `bg` is being set to None,
    /// the old color will be used for either foreground or background respectively.
    fn set_default_color(&mut self, color: ColorPair);
    /// Writes text string to the terminal at current position using default color. The cursor is
    /// moved after the text written, or at the end position in the terminal if there's no room
    /// left.
    fn write(&mut self, s: &str) {
        let color = self.get_default_color();
        self.write_in_color(s, color);
    }
    /// Writes text string to the terminal at current position using given color. The cursor is
    /// moved after the text written, or at the end position in the terminal if there's no room
    /// left.
    fn write_in_color(&mut self, s: &str, color: ColorPair) {
        self.write_colored(s, repeat(color));
    }
    /// Writes text string to the terminal at current position, with colors of consecutive
    /// characters taken from the iterator. If there are less colors in the iterator than letters
    /// in the text, the output will be truncated. The cursor is moved after the text written, or at
    /// the end position in the terminal if there's no room left.
    fn write_colored<T>(&mut self, s: &str, colors: T) where T: Iterator<Item=ColorPair>;
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::SystemTerminal;
#[cfg(windows)]
pub use self::windows::init_terminal;

#[cfg(unix)]
mod ncurses;
#[cfg(unix)]
pub use self::ncurses::SystemTerminal;
#[cfg(unix)]
pub use self::ncurses::init_terminal;
