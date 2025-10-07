mod args;

use clap::Parser;
use anyhow::Result;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::Pod;
use std::process::Command;
use kube::api::ListParams;
use kube::runtime::reflector::Lookup;
use crate::args::Args;

/// kubectl plugin to get logs by container index
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let namespace: &str = args.namespace
        .as_deref()
        .unwrap_or("default");


    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, namespace);

    let pod_list = find_matching_pods(pods, &args.pod_part).await.expect("Failed to find matching pods");
    
    let pod = pod_list
        .get(args.index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Pod not found"))?;

    let pod_name = &pod.name().ok_or_else(|| anyhow::anyhow!("Pod name not found"))?;

    let mut cmd = Command::new("kubectl");

    cmd.args(["logs", pod_name]);

    if namespace != "default" {
        cmd.args(["-n", namespace]);
    }

    if args.follow {
        cmd.arg("-f");
    }

    // if args.tail > 0 {
    //     cmd.arg("-t")
    //         .arg(args.tail.to_string()
    //             .as_str());
    // }

    cmd
        .status()?;

    Ok(())
}

pub async fn find_matching_pods(
    pods: Api<Pod>,
    partial: &str,
) -> Result<Vec<Pod>, Box<dyn std::error::Error>> {
    let pod_list = pods.list(&ListParams::default()).await?;

    let matches: Vec<Pod> = pod_list.items
        .into_iter()
        .filter(|pod| {
            pod.metadata.name
                .as_ref()
                .map(|name| name.contains(partial))
                .unwrap_or(false)
        })
        .collect();

    Ok(matches)
}
