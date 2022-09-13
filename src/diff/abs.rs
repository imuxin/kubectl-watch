use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::DynamicObject as KubeDynamicObject;
use kube::core::metadata::TypeMeta;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DynamicObject {
    /// The type fields, not always present
    #[serde(flatten, default)]
    pub types: Option<TypeMeta>,
    /// Object metadata
    pub metadata: ObjectMeta,

    /// All other keys
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl DynamicObject {
    pub fn from(o: &KubeDynamicObject) -> Self {
        let yaml = serde_yaml::to_string(o).unwrap();
        serde_yaml::from_str::<DynamicObject>(yaml.as_str()).unwrap()
    }
}

impl DynamicObject {
    pub fn exclude_types(&mut self) {
        self.types = None;
    }

    pub fn exclude_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}
