use anyhow::Result;

mod diff;
mod kube;
mod options;
mod output;
mod persistent;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let app: options::App = clap::Parser::parse();

    match kube::watch(&app).await {
        Ok(rx) => match app.mode {
            options::Mode::TUI => output::tui_print_process(&app, rx).await?,
            options::Mode::Simple => output::simple_print_process(rx).await?,
        },
        Err(error) => {
            panic!("{}", error)
        }
    }

    std::process::exit(0)
}
