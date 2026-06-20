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
use tokio::time::{sleep, Duration, Instant};

/// kubectl plugin to get logs by container index
#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(Args::parse());

    let namespace = args.namespace.clone().unwrap_or_else(|| "default".to_string());

    let client = Client::try_default().await?;
    let pods: Api<Pod> = Api::namespaced(client, namespace.as_str());

    let pod_list = find_matching_pods_with_wait(
        pods,
        &args.pod_part,
        args.wait_for_pods,
        args.all_pods,
        &args.index,
        &args.positional_indices,
    )
    .await?;

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
        |s| s.bright_green().to_string(),
        |s| s.blue().to_string(),
        |s| s.purple().to_string(),
        |s| s.bright_blue().to_string(),
    ];

    let mut handles = Vec::new();
    for (i, pod_name) in pod_names.iter().enumerate() {
        let pod_name = pod_name.clone();
        let namespace = namespace.clone();
        let args = Arc::clone(&args);

        let colorize = if should_color(&args.color) {
            colorizers[i % colorizers.len()]
        } else {
            |s: &str| s.to_string()
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
        cmd.arg("--tail")
            .arg(args.tail.unwrap_or(1000)
                .to_string()
                .as_str());
    }

    let mut child = cmd.stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let patterns: Vec<String> = args.match_text
        .iter()
        .map(|p| p.to_lowercase())
        .collect();

    for line in reader.lines() {
        match line {
            Ok(text) => {
                let text_lower = text.to_lowercase();
                let is_match = patterns.len() == 0 || patterns.iter().any(|p| text_lower.contains(p));

                if args.invert_match ^ is_match {
                    if text.contains("ERROR") {
                        println!("{}", text.red());
                    } else if text.contains("WARN") {
                        println!("{}", text.yellow());
                    } else {
                        println!("{}", colorize(&format!("[{}] {}", pod_name, text)));
                    }
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
) -> Result<Vec<Pod>> {
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

async fn find_matching_pods_with_wait(
    pods: Api<Pod>,
    partial: &str,
    wait_for_pods: Option<u64>,
    all_pods: bool,
    index: &Option<Vec<usize>>,
    positional_indices: &[usize],
) -> Result<Vec<Pod>> {
    let required_pod_count = required_pod_count(all_pods, index, positional_indices);

    if let Some(wait_for_pods) = wait_for_pods {
        let deadline = Instant::now() + Duration::from_secs(wait_for_pods);

        loop {
            let matches = find_matching_pods(pods.clone(), partial).await?;
            if matches.len() >= required_pod_count {
                return Ok(matches);
            }

            if Instant::now() >= deadline {
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }

        let matches = find_matching_pods(pods, partial).await?;
        if matches.len() >= required_pod_count {
            Ok(matches)
        } else {
            Err(anyhow::anyhow!(
                "Timed out waiting {} seconds for {} matching pod(s); found {}",
                wait_for_pods,
                required_pod_count,
                matches.len()
            ))
        }
    } else {
        let matches = find_matching_pods(pods, partial).await?;
        if matches.len() >= required_pod_count {
            Ok(matches)
        } else {
            Err(anyhow::anyhow!(
                "Found {} matching pod(s), but {} required",
                matches.len(),
                required_pod_count
            ))
        }
    }
}

fn required_pod_count(all_pods: bool, index: &Option<Vec<usize>>, positional_indices: &[usize]) -> usize {
    if all_pods {
        1
    } else {
        index
            .iter()
            .flatten()
            .chain(positional_indices.iter())
            .copied()
            .max()
            .map(|max_index| max_index + 1)
            .unwrap_or(1)
    }
}

fn should_color(color: &ColorChoice) -> bool {
    match color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => atty::is(Stream::Stdout),
    }
}