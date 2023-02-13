use terminal_size;

pub fn detect_display_width() -> usize {
    // terminal_size is actively maintained, but only considers
    // stdout. This is a problem inside git, where stderr is a TTY
    // with a size but stdout is piped to less.
    //
    // https://github.com/eminence/terminal-size/issues/23
    if let Some(width) = terminal_size::terminal_size().map(|(w, _)| w.0 as usize) {
        return width;
    }

    // term_size is no longer maintained, but it checks all of stdin,
    // stdout and stderr, so gives better results in may cases.
    if let Some(width) = term_size::dimensions().map(|(w, _)| w) {
        return width;
    }

    80
}
