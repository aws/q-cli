#![warn(rust_2018_idioms, future_incompatible)]
#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

pub mod ansi;
pub mod event;
pub mod grid;
pub mod index;
pub mod term;

pub use crate::grid::Grid;
pub use crate::term::Term;
