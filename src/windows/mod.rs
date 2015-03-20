use terminal::Terminal;

pub trait Window {
    fn size(&self) -> (u16, u16);
    fn draw<T: ?Sized>(&self, terminal: &mut T, x: i16, y: i16) where T: Terminal;
}
