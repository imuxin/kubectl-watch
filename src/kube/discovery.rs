use crate::kube::apigroup::{AllResource, ApiCapabilities, ApiGroup, ApiResource};

use anyhow::Result;
use kube::{
    api::{Api, DynamicObject},
    discovery::Scope,
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

impl DiscoveryMode {
    fn is_queryable(&self, group: &String) -> bool {
        match &self {
            Self::Allow(allowed) => allowed.contains(group),
            Self::Block(blocked) => !blocked.contains(group),
        }
    }
}

pub struct Discovery {
    client: Client,
    groups: HashMap<String, ApiGroup>,
    mode: DiscoveryMode,
}

impl Discovery {
    /// Construct a caching api discovery client
    #[must_use]
    pub fn new(client: Client) -> Self {
        let groups = HashMap::new();
        let mode = DiscoveryMode::Block(vec![]);
        Self {
            client,
            groups,
            mode,
        }
    }

    /// Returns iterator over all served groups
    pub fn groups(&self) -> impl Iterator<Item = &ApiGroup> {
        self.groups.values()
    }

    pub async fn run(mut self) -> Result<Self> {
        self.groups.clear();
        let api_groups = self.client.list_api_groups().await?;
        // query regular groups + crds under /apis
        for g in api_groups.groups {
            let key = g.name.clone();
            if self.mode.is_queryable(&key) {
                let apigroup = ApiGroup::query_apis(&self.client, g).await?;
                self.groups.insert(key, apigroup);
            }
        }
        // query core versions under /api
        let corekey = ApiGroup::CORE_GROUP.to_string();
        if self.mode.is_queryable(&corekey) {
            let coreapis = self.client.list_core_api_versions().await?;
            let apigroup = ApiGroup::query_core(&self.client, coreapis).await?;
            self.groups.insert(corekey, apigroup);
        }
        Ok(self)
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
        .flat_map(|group| {
            group
                .recommended_resources()
                .into_iter()
                .map(move |res| (group, res))
        })
        .filter(|(_, (res, _))| {
            println!("{} {}", res.kind.clone(), res.version.clone());
            let is_in_short_names = if let Some(short_names) = &res.short_names.clone() {
                short_names.contains(&name.to_owned())
            } else {
                false
            };
            name.eq_ignore_ascii_case(&res.kind)
                || name.eq_ignore_ascii_case(&res.plural)
                || is_in_short_names
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
        Api::all_with(client, &ar.to_kube_ar())
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar.to_kube_ar())
    } else {
        Api::default_namespaced_with(client, &ar.to_kube_ar())
    }
}

pub async fn new(cli: &Client) -> Result<Discovery> {
    let discovery = Discovery::new(cli.clone()).run().await?;
    Ok(discovery)
}
