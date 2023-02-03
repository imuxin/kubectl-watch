mod event;
mod tui;

use crate::diff;
use crate::options;

use k8s_openapi::{
    apimachinery::pkg::apis::meta::v1::Time,
    chrono::{Duration, Utc},
};
use kube::api::{DynamicObject, ResourceExt};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub async fn simple_print_process(mut rx: Receiver<DynamicObject>) -> std::io::Result<()> {
    println!("{0:<width$} {1:<20}", "NAME", "AGE", width = 63);
    while let Some(obj) = rx.recv().await {
        let age = format_creation_since(obj.creation_timestamp());
        println!("{0:<width$} {1:<20}", obj.name_any(), age, width = 63);
    }
    Ok(())
}

pub async fn delta_print_process(
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

pub async fn tui_print_process(mut rx: Receiver<DynamicObject>) -> anyhow::Result<()> {
    let (sender, receiver) = event::new_chan();
    let sender2 = sender.clone();

    tokio::spawn(async move {
        while let Some(obj) = rx.recv().await {
            sender.send(event::Msg::Obj(obj)).await.unwrap();
        }
    });

    tokio::spawn(async move {
        loop {
            if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                sender2.send(event::Msg::Key(key)).await.unwrap();
            }
        }
    });

    // draw terminal ui
    tui::main_tui(receiver).await
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
