use crate::diff::pipeline;
use crate::diff::Diff;
use crate::diff::Process;
use crate::kube::dynamic_object;
use crate::persistent;
use difft_lib::{diff_file, options, print_diff_result};
use kube::api::DynamicObject;
use std::path::PathBuf;
use tui::widgets::Paragraph;

pub struct Difft {
    include_managed_fields: bool,
}

impl Difft {
    pub fn new(include_managed_fields: bool) -> Self {
        Difft {
            include_managed_fields: include_managed_fields,
        }
    }
}

impl<'a> Diff<'a> for Difft {
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

    #[allow(unused_variables)]
    fn tui_diff(
        &mut self,
        pre: &DynamicObject,
        next: &DynamicObject,
    ) -> (Paragraph<'a>, Paragraph<'a>) {
        let mut p = pipeline::Pipeline::init();
        if !self.include_managed_fields {
            p.add_task(pipeline::exclude_managed_fields);
        }

        let mut l = dynamic_object::DynamicObject::from(pre);
        let mut r = dynamic_object::DynamicObject::from(next);

        p.process(&mut l, &mut r);

        // init delta args
        let (minus_file, plus_file) = persistent::tmp_store(&l, &r);
        // TODO: difft diff here

        (
            Paragraph::new("left").alignment(tui::layout::Alignment::Left),
            Paragraph::new("right").alignment(tui::layout::Alignment::Left),
        )
    }
}
