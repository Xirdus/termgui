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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ColorPair {
    pub fg: Option<Color>,
    pub bg: Option<Color>
}

impl ColorPair {
    pub fn compose(self, other: ColorPair) -> ColorPair {
        ColorPair {
            fg: self.fg.or(other.fg),
            bg: self.bg.or(other.bg)
        }
    }
}
