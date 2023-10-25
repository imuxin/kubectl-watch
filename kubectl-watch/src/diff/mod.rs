mod difft;
mod pipeline;

use self::pipeline::Process;
use crate::options;

use kube::api::DynamicObject;
use ratatui::widgets::Paragraph;
use std::path::PathBuf;

pub trait Diff<'a> {
    fn diff(&mut self, minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32>;
    fn tui_diff(
        &mut self,
        pre: Option<&DynamicObject>,
        cur: &DynamicObject,
    ) -> (Paragraph<'a>, Paragraph<'a>);
}

pub fn new<'a>(app: &options::App) -> Box<dyn Diff<'a>> {
    Box::new(difft::Difft::new(app.include_managed_fields))
}
