#[derive(clap::ArgEnum, Clone, PartialEq, Eq)]
pub enum Mode {
    TUI,
    Expand,
    Simple,
}
impl Default for Mode {
    fn default() -> Self {
        Self::TUI
    }
}

#[derive(clap::ArgEnum, Clone, PartialEq, Eq)]
pub enum DiffTool {
    Delta,
    Difft,
}
impl Default for DiffTool {
    fn default() -> Self {
        Self::Difft
    }
}

#[derive(clap::Parser)]
#[clap(version, about, long_about = None)]
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

    /// delta changes view mode
    #[clap(long, arg_enum, default_value_t)]
    pub mode: Mode,

    /// Diff tool to analyze delta changes
    #[clap(long, arg_enum, default_value_t)]
    pub diff_tool: DiffTool,

    /// Use tls to request api-server
    #[clap(long)]
    pub use_tls: bool,

    /// Set ture to show managed fields delta changes
    #[clap(long)]
    pub include_managed_fields: bool,

    /// A path, where all watched resources will be strored
    #[clap(long)]
    pub export: Option<String>,

    /// Support resource 'plural', 'kind' and 'shortname'
    pub resource: Option<String>,
    /// Resource name, optional
    pub name: Option<String>,
}
