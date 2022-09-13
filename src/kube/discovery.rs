use anyhow::Result;
use kube::{
    api::{Api, DynamicObject},
    discovery::{ApiCapabilities, ApiResource, Discovery as KubeDiscovery, Scope},
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

pub fn resolve_api_resource(
    discovery: &Discovery,
    name: &str,
) -> Option<(ApiResource, ApiCapabilities)> {
    // iterate through groups to find matching kind/plural names at recommended versions
    // and then take the minimal match by group.name (equivalent to sorting groups by group.name).
    // this is equivalent to kubectl's api group preference
    discovery
        .groups()
        .flat_map(|group| group.resources().into_iter().map(move |res| (group, res)))
        .filter(|(_, (res, _))| {
            name.eq_ignore_ascii_case(&res.kind) || name.eq_ignore_ascii_case(&res.plural)
        })
        .min_by_key(|(group, _res)| group.name())
        .map(|(_, res)| res)
}

pub fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: Client,
    ns: &Option<String>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}

pub async fn new_discovery(cli: &Client) -> Result<Discovery> {
    // discovery (to be able to infer apis from kind/plural only)
    let discovery = KubeDiscovery::new(cli.clone()).run().await?;
    // discovery2 will return all resources
    let discovery2: Discovery = unsafe { std::mem::transmute(discovery) };
    Ok(discovery2)
}
