mod delta;
mod difft;

use colored::*;
use kube::api::DynamicObject;
use kube::ResourceExt;
use std::env::temp_dir;
use std::fs;
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

pub fn diff(v: &Vec<DynamicObject>, diff_tool: &DiffTool) -> std::io::Result<i32> {
    if v.len() < 2 {
        return Ok(0);
    }

    paint_header_line(v.last().unwrap());

    // init delta args
    let (minus_file, plus_file) = store_to_file(v);

    new(diff_tool).diff(minus_file, plus_file)
}

fn paint_header_line(obj: &DynamicObject) {
    let api_version = &obj.types.as_ref().unwrap().api_version;
    let kind = &obj.types.as_ref().unwrap().kind;
    let namespace = &obj.namespace().unwrap();
    let name = &obj.name_any();
    let header_line = format!(
        "{} {} {} {} {} {} {} {}",
        "Apiversion", &api_version, "Kind:", kind, "Namespace:", namespace, "Name:", name,
    );
    let header_line_with_color = format!(
        "{} {} {} {} {} {} {} {}",
        "Apiversion",
        api_version.bright_yellow(),
        "Kind:",
        kind.bright_yellow(),
        "Namespace:",
        namespace.bright_yellow(),
        "Name:",
        name.bright_yellow(),
    );

    let count = header_line.chars().count();
    println!(
        "{}{}",
        "─".repeat(count + 1).bright_blue(),
        "┐".bright_blue()
    );
    println!("{} {}", &header_line_with_color, "│".bright_blue());
    println!(
        "{}{}",
        "─".repeat(count + 1).bright_blue(),
        "┘".bright_blue()
    );
}

fn store_to_file(v: &Vec<DynamicObject>) -> (PathBuf, PathBuf) {
    let obj_last = v.last().unwrap();
    let obj_penultimate = &v[v.len() - 2];

    let plus_yaml = serde_yaml::to_string(obj_last).unwrap();
    let minus_yaml = serde_yaml::to_string(obj_penultimate).unwrap();

    let mut path = temp_dir();
    path.push("kubectl-watch");
    fs::create_dir_all(&path).unwrap();
    let mut minus_file = path.clone();
    minus_file.push("minus.yaml");
    let mut plus_file = path.clone();
    plus_file.push("plus.yaml");

    std::fs::write(&minus_file, minus_yaml).unwrap();
    std::fs::write(&plus_file, plus_yaml).unwrap();

    let minus_file = PathBuf::from(&minus_file);
    let plus_file = PathBuf::from(&plus_file);
    return (minus_file, plus_file);
}
