#[macro_use]
extern crate log;

extern crate pretty_env_logger;

mod constants;
mod diff;
mod display;
mod files;
mod line_parser;
mod lines;
mod parse;
mod positions;
mod summary;

pub mod mainfn;
pub mod options;
pub use mainfn::{diff_file, print_diff_result, tui_diff_result, FgColor};
