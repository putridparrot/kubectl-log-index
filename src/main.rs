mod args;

use std::io::{BufRead, BufReader};
use clap::Parser;
use anyhow::Result;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::Pod;
use std::process::{Command};
use std::sync::Arc;
use colored::Colorize;
use kube::api::ListParams;
use kube::runtime::reflector::Lookup;
use crate::args::{Args, ColorChoice};
use std::thread;
use atty::Stream;

/// kubectl plugin to get logs by container index
#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(Args::parse());

    let namespace = args.namespace.clone().unwrap_or_else(|| "default".to_string());

    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());

    let pod_list = find_matching_pods(pods, &args.pod_part).await.expect("Failed to find matching pods");

    let indices = args.index.clone().unwrap_or_default()
        .into_iter()
        .chain(args.positional_indices.clone())
        .collect::<Vec<_>>();

    let pods: Vec<Pod> = if args.all_pods {
        pod_list.clone()
    } else {
        indices.iter()
            .map(|&i| {
                pod_list.get(i)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Pod index {} not found", i))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    let pod_names: Vec<String> = pods
        .iter()
        .map(|p| {
            p.name()
                .map(|name| name.to_string())
                .ok_or_else(|| anyhow::anyhow!("Pod name not found"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let colorizers: Vec<fn(&str) -> String> = vec![
        |s| s.green().to_string(),
        |s| s.bright_white().to_string(),
        |s| s.cyan().to_string(),
        |s| s.magenta().to_string(),
        |s| s.blue().to_string(),
        |s| s.purple().to_string(),
    ];

    let mut handles = Vec::new();
    for (i, pod_name) in pod_names.iter().enumerate() {
        let pod_name = pod_name.clone();
        let namespace = namespace.clone();
        let args = Arc::clone(&args);

        let colorize = if should_color(&args.color) {
            colorizers[i % colorizers.len()]
        } else {
            |s: &str| s.white().to_string()
        };

        let handle = thread::spawn(move || {
            fetch_logs_for_pod(&args, &namespace, &pod_name, colorize)
                .expect("Failed to fetch logs");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    Ok(())
}

fn fetch_logs_for_pod(args: &Args, namespace: &str, pod_name: &str,
                      colorize: fn(&str) -> String) -> Result<()> {
    let mut cmd = Command::new("kubectl");

    cmd.args(["logs", pod_name]);

    if namespace != "default" {
        cmd.args(["-n", namespace]);
    }

    if args.follow {
        cmd.arg("-f");
    }

    if args.tail.is_some() {
        cmd.arg("-t")
            .arg(args.tail.unwrap_or(1000)
                .to_string()
                .as_str());
    }

    let mut child = cmd.stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(text) => {
                if text.contains("ERROR") {
                    println!("{}", text.red());
                } else if text.contains("WARN") {
                    println!("{}", text.yellow());
                } else {
                    println!("{}", colorize(&format!("[{}] {}", pod_name, text)));
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
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

fn should_color(color: &ColorChoice) -> bool {
    match color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => atty::is(Stream::Stdout),
    }
}