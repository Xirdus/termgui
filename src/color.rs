//! Types for color handling.

/// Standard 16 colors of the terminal. The actual color displayed may vary.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    DarkBlack,
    DarkRed,
    DarkGreen,
    DarkBlue,
    DarkCyan,
    DarkMagenta,
    DarkYellow,
    DarkWhite,
    LightBlack,
    LightRed,
    LightGreen,
    LightBlue,
    LightCyan,
    LightMagenta,
    LightYellow,
    LightWhite
}

/// Holds the foreground (text) color and background color. "None" means the color will be inherited
/// from parent window.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ColorPair {
    /// Foreground color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>
}

impl ColorPair {
    /// Produces a copy of `self`, but with missing colors taken from `other`.
    pub fn compose(self, other: ColorPair) -> ColorPair {
        ColorPair {
            fg: self.fg.or(other.fg),
            bg: self.bg.or(other.bg)
        }
    }
}
