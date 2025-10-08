use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum ColorChoice {
    Always,
    Auto,
    Never,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "kubectl-log-index")]
#[command(author, version, about)]
pub struct Args {
    /// Partial name of the pod to match
    #[arg()]
    pub pod_part: String,
    /// Indices of the pod (0-based)
    #[arg(short, long)]
    pub index: Option<Vec<usize>>,
    #[arg()]
    pub positional_indices: Vec<usize>,
    /// Follow the log stream
    #[arg(short = 'f', long)]
    pub follow: bool,
    /// Lines from the end of the logs to show
    #[arg(long)]
    pub tail: Option<i64>,
    /// Kubernetes namespace (optional)
    #[arg(short, long)]
    pub namespace: Option<String>,
    /// Control colored output: always, auto, or never
    #[arg(long, default_value = "auto")]
    pub color: ColorChoice,
}
