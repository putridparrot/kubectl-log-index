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
    /// Merge all pod logs into a single stream 
    #[arg(short='a', long="all-pods")]
    pub all_pods: bool,
    /// Show only lines with matching text
    #[arg(long="match")]
    pub match_text: Option<String>,
    /// Show only lines not matching text
    #[arg(long="invert-match")]
    pub invert_match: bool,
}
