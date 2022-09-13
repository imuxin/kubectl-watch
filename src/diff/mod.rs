mod abs;
mod delta;
mod difft;
mod file;
mod pipeline;
mod utils;

use self::pipeline::Process;
use crate::options;
use colored::*;
use kube::api::DynamicObject;
use kube::ResourceExt;
use std::path::PathBuf;

#[derive(clap::ArgEnum, Clone, PartialEq, Eq)]
pub enum DiffTool {
    Delta,
    Difft,
}
impl Default for DiffTool {
    fn default() -> Self {
        Self::Delta
    }
}

pub trait Diff {
    fn diff(&mut self, minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32>;
}

fn new(diff_tool: &DiffTool) -> Box<dyn Diff> {
    match diff_tool {
        DiffTool::Delta => Box::new(delta::Delta::new()),
        DiffTool::Difft => Box::new(difft::Difft::new()),
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

    let mut l = abs::DynamicObject::from(&v[v.len() - 2]);
    let mut r = abs::DynamicObject::from(v.last().unwrap());

    p.process(&mut l, &mut r);

    // init delta args
    let (minus_file, plus_file) = file::store_to_file(&l, &r);

    new(&app.diff_tool).diff(minus_file, plus_file)
}

fn paint_header_line(v: &Vec<DynamicObject>) {
    let obj = v.last().unwrap();
    let api_version = &obj.types.as_ref().unwrap().api_version;
    let kind = &obj.types.as_ref().unwrap().kind;
    let namespace = &obj.namespace().unwrap();
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
        (v.len() - 1).to_string().bright_yellow(),
    );

    let seperator_line = "─".repeat(utils::detect_display_width());

    if v.len() > 2 {
        println!("{}", "\n".repeat(5));
    }
    println!("{}", seperator_line.bright_blue());
    println!("{}", &header_line_with_color);
    println!("{}", seperator_line.bright_blue());
}
