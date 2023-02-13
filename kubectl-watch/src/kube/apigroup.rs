use itertools::Itertools;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{
    APIGroup, APIResource, APIResourceList, APIVersions,
};
use kube::{
    core::{
        gvk::{GroupVersion, ParseGroupVersionError},
        ApiResource as KubeApiResource, Version,
    },
    discovery::Scope,
    Client, Result,
};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

/// Creates an `ApiResource` from a `meta::v1::APIResource` instance + its groupversion.
///
/// Returns a `DiscoveryError` if the passed group_version cannot be parsed
pub(crate) fn parse_apiresource(
    ar: &APIResource,
    group_version: &str,
) -> Result<ApiResource, ParseGroupVersionError> {
    let gv: GroupVersion = group_version.parse()?;
    // NB: not safe to use this with subresources (they don't have api_versions)
    Ok(ApiResource {
        group: ar.group.clone().unwrap_or_else(|| gv.group.clone()),
        version: ar.version.clone().unwrap_or_else(|| gv.version.clone()),
        api_version: gv.api_version(),
        kind: ar.kind.to_string(),
        plural: ar.name.clone(),
        short_names: ar.short_names.clone(),
    })
}

/// Creates `ApiCapabilities` from a `meta::v1::APIResourceList` instance + a name from the list.
///
/// Returns a `DiscoveryError` if the list does not contain resource with passed `name`.
pub(crate) fn parse_apicapabilities(list: &APIResourceList, name: &str) -> Result<ApiCapabilities> {
    let ar = list
        .resources
        .iter()
        .find(|r| r.name == name)
        .ok_or_else(|| panic!("{:?}", "missing resource"))?;
    let scope = if ar.namespaced {
        Scope::Namespaced
    } else {
        Scope::Cluster
    };

    let subresource_name_prefix = format!("{}/", name);
    let mut subresources = vec![];
    for res in &list.resources {
        if let Some(subresource_name) = res.name.strip_prefix(&subresource_name_prefix) {
            let mut api_resource = parse_apiresource(res, &list.group_version)
                .map_err(|ParseGroupVersionError(s)| panic!("invalid group version: {}", s))?;
            api_resource.plural = subresource_name.to_string();
            let caps = parse_apicapabilities(list, &res.name)?; // NB: recursion
            subresources.push((api_resource, caps));
        }
    }
    Ok(ApiCapabilities {
        scope,
        subresources,
        operations: ar.verbs.clone(),
    })
}

/// Information about a Kubernetes API resource
///
/// Enough information to use it like a `Resource` by passing it to the dynamic `Api`
/// constructors like `Api::all_with` and `Api::namespaced_with`.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiResource {
    /// Resource group, empty for core group.
    pub group: String,
    /// group version
    pub version: String,
    /// apiVersion of the resource (v1 for core group,
    /// groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// Plural name of the resource
    pub plural: String,
    /// Short names of the resource
    pub short_names: Option<Vec<String>>,
}

impl ApiResource {
    pub fn to_kube_ar(self) -> KubeApiResource {
        KubeApiResource {
            group: self.group,
            version: self.version,
            api_version: self.api_version,
            kind: self.kind,
            plural: self.plural,
        }
    }
}

/// Contains the capabilities of an API resource
#[derive(Debug, Clone)]
pub struct ApiCapabilities {
    /// Scope of the resource
    pub scope: Scope,
    /// Available subresources.
    ///
    /// Please note that returned ApiResources are not standalone resources.
    /// Their name will be of form `subresource_name`, not `resource_name/subresource_name`.
    /// To work with subresources, use `Request` methods for now.
    pub subresources: Vec<(ApiResource, ApiCapabilities)>,
    /// Supported operations on this resource
    pub operations: Vec<String>,
}

pub(crate) struct GroupVersionData {
    /// Pinned api version
    pub(crate) version: String,
    /// Pair of dynamic resource info along with what it supports.
    pub(crate) resources: Vec<(ApiResource, ApiCapabilities)>,
}

impl GroupVersionData {
    /// Given an APIResourceList, extract all information for a given version
    pub(crate) fn new(version: String, list: APIResourceList) -> Result<Self> {
        let mut resources = vec![];
        for res in &list.resources {
            // skip subresources
            if res.name.contains('/') {
                continue;
            }
            // NB: these two should be infallible from discovery when k8s api is well-behaved, but..
            let ar = parse_apiresource(res, &list.group_version)
                .map_err(|ParseGroupVersionError(s)| panic!("invalid group version: {}", s))?;
            let caps = parse_apicapabilities(&list, &res.name)?;
            resources.push((ar, caps));
        }
        Ok(GroupVersionData { version, resources })
    }
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

pub trait AllResource {
    fn recommended_resources(&self) -> Vec<(ApiResource, ApiCapabilities)>;
}

impl ApiGroup {
    /// Core group name
    pub const CORE_GROUP: &'static str = "";

    /// Returns the name of this group.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AllResource for ApiGroup {
    fn recommended_resources(&self) -> Vec<(ApiResource, ApiCapabilities)> {
        self.data
            .iter()
            .map(|gvd| gvd.resources.clone())
            .concat()
            .iter()
            .into_group_map_by(|(ar, _)| ar.kind.clone())
            .into_iter()
            .map(|(_, mut v)| {
                v.sort_by_cached_key(|(ar, _)| {
                    Reverse(Version::parse(ar.version.as_str()).priority())
                });
                v[0].to_owned()
            })
            .collect()
    }
}

impl ApiGroup {
    pub(crate) async fn query_apis(client: &Client, g: APIGroup) -> Result<Self> {
        tracing::debug!(name = g.name.as_str(), "Listing group versions");
        let key = g.name;
        if g.versions.is_empty() {
            panic!("{:?}", "empty group version");
        }
        let mut data = vec![];
        for vers in &g.versions {
            let resources = client.list_api_group_resources(&vers.group_version).await?;
            data.push(GroupVersionData::new(vers.version.clone(), resources)?);
        }
        let mut group = ApiGroup {
            name: key,
            data,
            preferred: g.preferred_version.map(|v| v.version),
        };
        group.sort_versions();
        Ok(group)
    }

    pub(crate) async fn query_core(client: &Client, coreapis: APIVersions) -> Result<Self> {
        let mut data = vec![];
        if coreapis.versions.is_empty() {
            panic!("{:?}", "empty api group");
        }
        for v in coreapis.versions {
            let resources = client.list_core_api_resources(&v).await?;
            data.push(GroupVersionData::new(v, resources)?);
        }
        let mut group = ApiGroup {
            name: ApiGroup::CORE_GROUP.to_string(),
            data,
            preferred: Some("v1".to_string()),
        };
        group.sort_versions();
        Ok(group)
    }

    fn sort_versions(&mut self) {
        self.data
            .sort_by_cached_key(|gvd| Reverse(Version::parse(gvd.version.as_str()).priority()))
    }
}
