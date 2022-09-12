use crate::diff::Diff;
use difft_lib::{diff_file, options, print_diff_result};
use std::path::PathBuf;

pub struct Difft {}

impl Difft {
    pub fn new() -> Self {
        Difft {}
    }
}

impl Diff for Difft {
    fn diff(&mut self, minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32> {
        let graph_limit = options::DEFAULT_GRAPH_LIMIT;
        let byte_limit = options::DEFAULT_BYTE_LIMIT;
        let display_options = options::DisplayOptions {
            background_color: options::BackgroundColor::Dark,
            use_color: true,
            print_unchanged: false,
            tab_width: options::DEFAULT_TAB_WIDTH,
            display_mode: options::DisplayMode::SideBySide,
            display_width: options::detect_display_width(),
            syntax_highlight: true,
            in_vcs: true,
        };
        let language_override = None;
        let missing_as_empty = false;
        let diff_result = diff_file(
            minus_file.to_str().unwrap(),
            plus_file.to_str().unwrap(),
            minus_file.as_path(),
            plus_file.as_path(),
            &display_options,
            missing_as_empty,
            graph_limit,
            byte_limit,
            language_override,
        );
        print_diff_result(&display_options, &diff_result);
        Ok(0)
    }
}
