mod apigroup;
pub mod client;
mod discovery;
pub mod dynamic_object;
mod watch;

pub use client::client;
pub use discovery::new;
pub use discovery::{dynamic_api, resolve_api_resource};
pub use watch::watch;
