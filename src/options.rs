use crate::diff;

#[derive(clap::Parser)]
pub struct App {
    /// Selector (label query) to filter on, supports '=', '==', and '!='.(e.g. -l key1=value1,key2=value2)
    #[clap(long, short = 'l')]
    pub selector: Option<String>,

    /// If present, the namespace scope for this CLI request
    #[clap(long, short)]
    pub namespace: Option<String>,

    /// If present, list the requested object(s) across all namespaces
    #[clap(long, short = 'A')]
    pub all: bool,

    /// Skip show delta changes view
    #[clap(long, short)]
    pub skip_delta: bool,

    /// Diff tool to analyze delta changes
    #[clap(long, arg_enum, default_value_t)]
    pub diff_tool: diff::DiffTool,

    /// Use tls to request api-server
    #[clap(long)]
    pub use_tls: bool,

    /// Set ture to show managed fields delta changes
    #[clap(long)]
    pub include_managed_fields: bool,

    pub resource: Option<String>,
    pub name: Option<String>,
}
