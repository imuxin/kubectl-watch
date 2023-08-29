use crate::kube::{client, discovery};
use crate::options;
use crate::persistent;

use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};

use kube::{
    api::{DynamicObject, ListParams},
    discovery::Scope,
    runtime::{watcher, WatchStreamExt},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub async fn watch(app: &options::App) -> Result<Receiver<DynamicObject>> {
    let cli = client::client(app.use_tls).await?;
    let discovery = discovery::new(&cli).await?;
    let resource = app.resource.clone();
    // Common discovery, parameters, and api configuration for a single resource
    let (ar, caps) = discovery::resolve_api_resource(&discovery, resource.as_str())
        .with_context(|| format!("resource {:?} not found in cluster", resource))?;

    if caps.scope == Scope::Cluster && !app.namespace.is_none() {
        panic!("{} is not a Namespaced-Resources!", resource);
    }

    let mut lp = ListParams::default();
    if let Some(label) = app.selector.clone() {
        lp = lp.labels(label.as_str());
    }

    if let Some(name) = app.name.clone() {
        lp = lp.fields(&format!("metadata.name={}", name));
    }
    let api = discovery::dynamic_api(ar, caps, cli, &app.namespace, app.all);

    tracing::info!(?resource, name = ?app.name.clone().unwrap_or_default(), "requested objects");

    let (tx, rx): (Sender<DynamicObject>, Receiver<DynamicObject>) = channel(32);

    let export_path = app.export.clone();

    tokio::spawn(async move {
        // present a dumb table for it for now. kubectl does not do this anymore.
        let mut stream = watcher(api, lp).applied_objects().boxed();
        loop {
            let obj = match stream.try_next().await {
                Ok(obj) => obj.unwrap(),
                Err(error) => {
                    panic!("failed to get stream response: {:?}", error)
                }
            };
            persistent::store_resource(&export_path, &obj);
            tx.send(obj).await.unwrap();
        }
    });

    return Ok(rx);
}
