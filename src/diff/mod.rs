mod delta;
mod difft;
mod pipeline;
mod utils;

use self::pipeline::Process;
use crate::kube::dynamic_object;
use crate::options;
use crate::persistent;

use colored::*;
use kube::api::DynamicObject;
use kube::ResourceExt;
use std::path::PathBuf;

pub trait Diff {
    fn diff(&mut self, minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32>;
}

fn new(diff_tool: &options::DiffTool) -> Box<dyn Diff> {
    match diff_tool {
        options::DiffTool::Delta => Box::new(delta::Delta::new()),
        options::DiffTool::Difft => Box::new(difft::Difft::new()),
    }
}

pub fn diff(app: &options::App, v: &Vec<DynamicObject>) -> std::io::Result<i32> {
    if v.len() < 2 {
        return Ok(0);
    }

    paint_header_line(v);

    let mut p = pipeline::Pipeline::init();

    if !app.include_managed_fields {
        p.add_task(pipeline::exclude_managed_fields);
    }

    let mut l = dynamic_object::DynamicObject::from(&v[v.len() - 2]);
    let mut r = dynamic_object::DynamicObject::from(v.last().unwrap());

    p.process(&mut l, &mut r);

    // init delta args
    let (minus_file, plus_file) = persistent::tmp_store(&l, &r);

    new(&app.diff_tool).diff(minus_file, plus_file)
}

fn paint_header_line(list: &Vec<DynamicObject>) {
    let obj = list.last().unwrap();
    let api_version = &obj.types.as_ref().unwrap().api_version;
    let kind = &obj.types.as_ref().unwrap().kind;
    let namespace = &obj.namespace().unwrap_or_default();
    let name = &obj.name_any();

    let header_line_with_color = format!(
        "{} {} {} {} {} {} {} {} --- Event Number: {}",
        "Apiversion",
        api_version.bright_yellow(),
        "Kind:",
        kind.bright_yellow(),
        "Namespace:",
        namespace.bright_yellow(),
        "Name:",
        name.bright_yellow(),
        (list.len() - 1).to_string().bright_yellow(),
    );

    let seperator_line = "─".repeat(utils::detect_display_width());

    if list.len() > 2 {
        println!("{}", "\n".repeat(5));
    }
    println!("{}", seperator_line.bright_blue());
    println!("{}", &header_line_with_color);
    println!("{}", seperator_line.bright_blue());
}
