use color::ColorPair;
use std::iter::repeat;

trivial_error! {
    TerminalError = "Couldn't initialize terminal";
    ResizeError = "Couldn't resize terminal";
    CursorError = "Couldn't set cursor to given position";
}

pub trait Terminal {
    fn size(&self) -> (u16, u16);
    fn resize(&mut self, u16, u16) -> Result<(),ResizeError>;
    fn get_cursor_pos(&self) -> (u16, u16);
    fn set_cursor_pos(&mut self, x: u16, y: u16) ->  Result<(),CursorError>;
    fn get_default_color(&self) -> ColorPair;
    fn set_default_color(&mut self, color: ColorPair);
    fn write(&mut self, s: &str) {
        let color = self.get_default_color();
        self.write_in_color(s, color);
    }
    fn write_in_color(&mut self, s: &str, color: ColorPair) {
        self.write_colored(s, repeat(color));
    }
    fn write_colored<T>(&mut self, s: &str, colors: T) where T: Iterator<Item=ColorPair>;
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::SystemTerminal;
#[cfg(windows)]
pub use self::windows::init_terminal;
