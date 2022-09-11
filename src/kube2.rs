use kube::{
    discovery::{ApiCapabilities, ApiResource},
    Client,
};
use std::collections::HashMap;

#[allow(dead_code)]
enum DiscoveryMode {
    /// Only allow explicitly listed apigroups
    Allow(Vec<String>),
    /// Allow all apigroups except the ones listed
    Block(Vec<String>),
}

#[allow(dead_code)]
pub struct Discovery {
    client: Client,
    groups: HashMap<String, ApiGroup>,
    mode: DiscoveryMode,
}

impl Discovery {
    /// Returns iterator over all served groups
    pub fn groups(&self) -> impl Iterator<Item = &ApiGroup> {
        self.groups.values()
    }
}

#[allow(dead_code)]
pub(crate) struct GroupVersionData {
    /// Pinned api version
    pub(crate) version: String,
    /// Pair of dynamic resource info along with what it supports.
    pub(crate) resources: Vec<(ApiResource, ApiCapabilities)>,
}

trait AllResource {
    fn resources(&self) -> Vec<(ApiResource, ApiCapabilities)>;
}

#[allow(dead_code)]
pub struct ApiGroup {
    /// Name of the group e.g. apiregistration.k8s.io
    name: String,
    /// List of resource information, capabilities at particular versions
    data: Vec<GroupVersionData>,
    //// Preferred version if exported by the `APIGroup`
    preferred: Option<String>,
}

impl ApiGroup {
    pub fn resources(&self) -> Vec<(ApiResource, ApiCapabilities)> {
        let mut r: Vec<(ApiResource, ApiCapabilities)> = vec![];
        for i in self.data.iter() {
            r.append(&mut i.resources.clone());
        }
        return r;
    }

    /// Returns the name of this group.
    pub fn name(&self) -> &str {
        &self.name
    }
}
