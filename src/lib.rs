//! A library for creating GUI in command line applications. Similar in concept to ncurses, but with
//! much more modern design.

#![cfg_attr(target_os="windows", feature(collections))]

pub mod color;
pub mod terminal;
