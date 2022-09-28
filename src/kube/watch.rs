use crate::diff;
use crate::kube::{client, discovery};
use crate::options;
use crate::persistent;

use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::{
    apimachinery::pkg::apis::meta::v1::Time,
    chrono::{Duration, Utc},
};
use kube::{
    api::{DynamicObject, ListParams, ResourceExt},
    discovery::Scope,
    runtime::{watcher, WatchStreamExt},
};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub async fn watch(app: options::App) -> Result<()> {
    let cli = match client::client(app.use_tls).await {
        Ok(cli) => cli,
        Err(error) => {
            panic!("failed to init kube client: {:?}", error)
        }
    };

    let discovery = discovery::new(&cli).await?;
    if let Some(resource) = app.resource.clone() {
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

        if !app.skip_delta {
            tokio::spawn(async move { delta_print_process(&app, rx).await });
        } else {
            tokio::spawn(async move { simple_print_process(rx).await });
        }

        // present a dumb table for it for now. kubectl does not do this anymore.
        let mut stream = watcher(api, lp).applied_objects().boxed();
        while let Some(obj) = stream.try_next().await? {
            persistent::store_resource(&export_path, &obj);
            tx.send(obj).await.unwrap();
        }
    }

    Ok(())
}

async fn simple_print_process(mut rx: Receiver<DynamicObject>) -> std::io::Result<()> {
    println!("{0:<width$} {1:<20}", "NAME", "AGE", width = 63);
    while let Some(obj) = rx.recv().await {
        let age = format_creation_since(obj.creation_timestamp());
        println!("{0:<width$} {1:<20}", obj.name_any(), age, width = 63);
    }
    Ok(())
}

async fn delta_print_process(
    app: &options::App,
    mut rx: Receiver<DynamicObject>,
) -> std::io::Result<()> {
    let mut map = HashMap::new();
    while let Some(obj) = rx.recv().await {
        let empty_list = Vec::<DynamicObject>::new();
        let name = obj.name_any();
        let namespace = obj.namespace().unwrap_or_default();
        let key = name + &namespace;
        map.entry(key.clone()).or_insert(empty_list);
        if let Some(list) = map.get_mut(&key.clone()) {
            list.push(obj);
            let exit_code = diff::diff(app, list)?;
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
