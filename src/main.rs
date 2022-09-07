use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::{
    apimachinery::pkg::apis::meta::v1::Time,
    chrono::{Duration, Utc},
};
use kube::{
    api::{Api, DynamicObject, ListParams, ResourceExt},
    discovery::{ApiCapabilities, ApiResource, Discovery, Scope},
    runtime::{watcher, WatchStreamExt},
    Client,
};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver, Sender};

mod diff;

#[derive(clap::Parser)]
struct App {
    /// Selector (label query) to filter on, supports '=', '==', and '!='.(e.g. -l key1=value1,key2=value2)
    #[clap(long, short = 'l')]
    selector: Option<String>,

    /// If present, the namespace scope for this CLI request
    #[clap(long, short)]
    namespace: Option<String>,

    /// If present, list the requested object(s) across all namespaces
    #[clap(long, short = 'A')]
    all: bool,

    /// Show delta changes view
    #[clap(long, short)]
    delta: bool,

    resource: Option<String>,
    name: Option<String>,
}

impl App {
    async fn watch(
        &self,
        api: Api<DynamicObject>,
        lp: ListParams,
        tx: Sender<DynamicObject>,
    ) -> Result<()> {
        // present a dumb table for it for now. kubectl does not do this anymore.
        let mut stream = watcher(api, lp).applied_objects().boxed();
        while let Some(inst) = stream.try_next().await? {
            tx.send(inst).await.unwrap();
        }
        Ok(())
    }
}

fn resolve_api_resource(
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
            // match on both resource name and kind name
            // ideally we should allow shortname matches as well
            name.eq_ignore_ascii_case(&res.kind) || name.eq_ignore_ascii_case(&res.plural)
        })
        .min_by_key(|(group, _res)| group.name())
        .map(|(_, res)| res)
}

fn dynamic_api(
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app: App = clap::Parser::parse();
    let client = Client::try_default().await?;

    // discovery (to be able to infer apis from kind/plural only)
    let discovery = Discovery::new(client.clone()).run().await?;

    if let Some(resource) = &app.resource {
        // Common discovery, parameters, and api configuration for a single resource
        let (ar, caps) = resolve_api_resource(&discovery, resource)
            .with_context(|| format!("resource {:?} not found in cluster", resource))?;
        let mut lp = ListParams::default();
        if let Some(label) = &app.selector {
            lp = lp.labels(label);
        }
        if let Some(name) = app.name.clone() {
            lp = lp.fields(&format!("metadata.name={}", name));
        }
        let api = dynamic_api(ar, caps, client, &app.namespace, app.all);

        tracing::info!(?resource, name = ?app.name.clone().unwrap_or_default(), "requested objects");

        let (tx, rx): (Sender<DynamicObject>, Receiver<DynamicObject>) = channel(32);
        if app.delta {
            tokio::spawn(async move { delta_print_process(rx).await });
        } else {
            tokio::spawn(async move { simple_print_process(rx).await });
        }

        app.watch(api, lp, tx).await?;
    }
    Ok(())
}

async fn simple_print_process(mut rx: Receiver<DynamicObject>) -> std::io::Result<()> {
    println!("{0:<width$} {1:<20}", "NAME", "AGE", width = 63);
    while let Some(inst) = rx.recv().await {
        let age = format_creation_since(inst.creation_timestamp());
        println!("{0:<width$} {1:<20}", inst.name_any(), age, width = 63);
    }
    Ok(())
}

async fn delta_print_process(mut rx: Receiver<DynamicObject>) -> std::io::Result<()> {
    let mut map = HashMap::new();
    while let Some(inst) = rx.recv().await {
        let v: Vec<DynamicObject> = Vec::new();
        let name = inst.name_any();
        let namespace = inst.namespace().unwrap();
        let key = name + &namespace;
        map.entry(key.clone()).or_insert(v);
        if let Some(obj_arr) = map.get_mut(&key.clone()) {
            obj_arr.push(inst);
            let exit_code = diff::diff(obj_arr)?;
            if exit_code != 0 && exit_code != 1 {
                std::process::exit(exit_code);
            }
        }
    }
    Ok(())
}

fn format_creation_since(time: Option<Time>) -> String {
    format_duration(Utc::now().signed_duration_since(time.unwrap().0))
}
fn format_duration(dur: Duration) -> String {
    match (dur.num_days(), dur.num_hours(), dur.num_minutes()) {
        (days, _, _) if days > 0 => format!("{}d", days),
        (_, hours, _) if hours > 0 => format!("{}h", hours),
        (_, _, mins) => format!("{}m", mins),
    }
}

// fn main() -> std::io::Result<()> {
//     let minus_file = PathBuf::from(r"/home/muxin/draft/minus_str");
//     let plus_file = PathBuf::from(r"/home/muxin/draft/plus_str");
//     let exit_code = diff::diff_files(minus_file, plus_file)?;
//     process::exit(exit_code);
// }
