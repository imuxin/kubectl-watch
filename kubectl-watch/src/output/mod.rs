mod db;
mod event;
mod tui;
mod utils;

use crate::options;
use kube::api::{DynamicObject, ResourceExt};
use tokio::sync::mpsc::Receiver;

pub async fn simple_print_process(mut rx: Receiver<DynamicObject>) -> std::io::Result<()> {
    println!("{0:<width$} {1:<20}", "NAME", "AGE", width = 63);
    while let Some(obj) = rx.recv().await {
        let age = utils::format_creation_since(obj.creation_timestamp());
        println!("{0:<width$} {1:<20}", obj.name_any(), age, width = 63);
    }
    Ok(())
}

pub async fn tui_print_process(
    app: &options::App,
    mut rx: Receiver<DynamicObject>,
) -> anyhow::Result<()> {
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
    tui::main_tui(app, receiver).await
}
