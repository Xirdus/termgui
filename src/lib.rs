#![feature(core)]

mod macros;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;
