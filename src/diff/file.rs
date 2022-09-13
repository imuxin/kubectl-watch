use crate::diff::abs;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

pub fn store_to_file(l: &abs::DynamicObject, r: &abs::DynamicObject) -> (PathBuf, PathBuf) {
    let l_yaml = serde_yaml::to_string(l).unwrap();
    let r_yaml = serde_yaml::to_string(r).unwrap();

    let mut path = temp_dir();
    path.push("kubectl-watch");
    fs::create_dir_all(&path).unwrap();
    let mut minus_file = path.clone();
    minus_file.push("minus.yaml");
    let mut plus_file = path.clone();
    plus_file.push("plus.yaml");

    std::fs::write(&minus_file, l_yaml).unwrap();
    std::fs::write(&plus_file, r_yaml).unwrap();

    let minus_file = PathBuf::from(&minus_file);
    let plus_file = PathBuf::from(&plus_file);
    return (minus_file, plus_file);
}
