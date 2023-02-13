use crossterm::event::KeyEvent;
use kube::api::DynamicObject;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Msg {
    Key(KeyEvent),
    Obj(DynamicObject),
}

pub fn new_chan() -> (mpsc::Sender<Msg>, mpsc::Receiver<Msg>) {
    mpsc::channel::<Msg>(32)
}
