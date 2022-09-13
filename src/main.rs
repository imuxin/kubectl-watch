use anyhow::Result;

mod diff;
mod kube;
mod options;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app: options::App = clap::Parser::parse();
    kube::watch(app).await
}
