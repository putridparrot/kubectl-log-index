# kubectl-log-index

kubectl-log-index is a kubectl plugin that allows you to log from a pod via a partial name and index. For example
you might have multiple pods with the same name running within a namespace.

You can ofcourse use

```aiignore
kubectl log pod echo-pod-h5dvf -n dev
```

But it can be a bit tedious to remember the exact name of the pod.

This plugin allows you to log a pod using

```aiignore
kubectl log index echo 3 -n dev
```

where kube is the pod has a partial name _echo_ and there are at least 4 pods with this partial name, hence 3 is the index of the pod we want to log.

# Installation

Build using

```aiignore
cargo build --release
```

Kubectl plugins are just executables named with the prefix kubectl-, placed in your $PATH, so you can copy the binary to a folder in your PATH, for example

```aiignore
mv target/release/kubectl-log-index /usr/local/bin/
```
To check if the plugin can be located, run 

```aiignore
kubectl plugin list
```

# Usage

Basic usage is

```aiignore
kubectl log index <pod-partial-name> <index> --namespace <namespace>
kubectl log index <pod-partial-name> <index> -n <namespace>
kubectl log index <pod-partial-name> <index> -n <namespace> -f --tail <lines>
```

We also have the option to merge logs from multiple pods with the same partial name

```aiignore
kubectl log index <pod-partial-name> <space-delimited-indicies> --namespace <namespace>
```

In such situations, by default the logs for each pod are coloured differently. If you want to merge the logs, you can use the --no-color option. 

```aiignore
kubectl log index <pod-partial-name> <space-delimited-indicies> --namespace <namespace> --color=auto
```

Colour options are auto, always, never. Auto is the default and will colour the logs if the output is a terminal.

We can also log all pods with the same partial name using the --all-pods (short form -a) option.

```aiignore
kubectl log index <pod-partial-name> --all-pods --namespace <namespace> --color=auto
```

# Windows

- Copy it to a folder in your PATH: Recommended locations:
- Or: C:\Users\<YourUsername>\AppData\Local\Microsoft\WindowsApps\
- Or: any folder already listed in your system’s PATH environment variable
