#![feature(core)]
#![feature(collections)]

mod macros;
mod color;

pub use color::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
