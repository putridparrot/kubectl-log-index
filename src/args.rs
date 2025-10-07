use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "kubectl-log-index")]
#[command(author, version, about)]
pub struct Args {
    /// Partial name of the pod to match
    pub pod_part: String,
    /// Index of the pod (0-based)
    pub index: usize,
    /// Follow the log stream
    #[arg(short = 'f', long)]
    pub follow: bool,
    /// Lines from the end of the logs to show
    #[arg(long, default_value_t = 100)]
    pub tail: i64,
    /// Kubernetes namespace (optional)
    #[arg(short, long)]
    pub namespace: Option<String>,
}
