use kube::api::DynamicObject;
use kube::api::ResourceExt;
use kube::Resource;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

pub fn tmp_store(l_yaml: String, r_yaml: String) -> (PathBuf, PathBuf) {
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

pub fn store_resource(path: &Option<String>, obj: &DynamicObject) {
    if let Some(path) = path {
        if path == "" {
            return;
        }

        let mut path_buf = PathBuf::from(path.as_str());
        if let Some(namespace) = obj.namespace() {
            path_buf.push(namespace);
        }
        path_buf.push(obj.name_any());
        fs::create_dir_all(&path_buf).unwrap();

        let file_name = format!("{}.yaml", obj.meta().resource_version.clone().unwrap());
        path_buf.push(file_name);

        let yaml = serde_yaml::to_string(obj).unwrap();
        std::fs::write(&path_buf, yaml).unwrap();
    }
}
