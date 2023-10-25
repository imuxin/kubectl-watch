# kubectl-watch

:tada::tada::tada: Since 0.2.3 we have **terminal UI**.

Another watch tool with visualization view of delta change for kubernetes resources.

![overview.gif](./assets/overview.gif)

## Installation

### Use docker image [recommend]

1. Docker should be preinstalled, more installation details please visit [official website](https://docs.docker.com/engine/install/).

2. Download kubectl-watch script into your $PATH folder
```bash
curl -SL# "https://github.com/imuxin/kubectl-watch/blob/master/script/kubectl-watch?raw=true" >> /usr/local/bin/kubectl-watch && chmod +x /usr/local/bin/kubectl-watch
```

### Download kubectl-watch from [release assets](https://github.com/imuxin/kubectl-watch/releases).

### Build and install from source using [Cargo](https://crates.io/crates/kubectl-watch):

```bash
cargo install kubectl-watch --locked
```

## Command help

```bash
USAGE:
    kubectl-watch [OPTIONS] [ARGS]

ARGS:
    <RESOURCE>    Support resource 'plural', 'kind' and 'shortname'
    <NAME>        Resource name, optional

OPTIONS:
    -A, --all                       If present, list the requested object(s) across all namespaces
        --export <EXPORT>           A path, where all watched resources will be strored
    -h, --help                      Print help information
        --include-managed-fields    Set ture to show managed fields delta changes
    -l, --selector <SELECTOR>       Selector (label query) to filter on, supports '=', '==', and '!='.(e.g. -l key1=value1,key2=value2)
        --mode <MODE>               delta changes view mode [default: tui] [possible values: tui, simple]
    -n, --namespace <NAMESPACE>     If present, the namespace scope for this CLI request
        --use-tls                   Use tls to request api-server
    -V, --version                   Print version information
```

## TUI keystroke help

| Keystroke                | Description                        |
| ------------------------ | ---------------------------------- |
| char "j" or Down Arrow ↓ | go to next  resource               |
| char "k" or Up Arrow ↑   | go to previous resource            |
| Enter ↵                  | Only show selected resource events |
| ESC                      | go back                            |
| PageUP                   | scroll up diff content             |
| PageDown                 | scroll down diff content           |
| Home                     | reset scroll                       |

## Examples

watch deploy in all namespace
```bash
kubectl-watch deployment -A
```

watch deploy on some namespace
```bash
kubectl-watch deployment -n {namespace}
```

export watched resources into local storage, just add `--export "/to/your/path"`
```bash
kubectl-watch {resource} --export "/to/your/path"
```

`managed-fields` will be excluded by default, add `--include-managed-fields` can show the managed fields changes.
```bash
kubectl-watch {resource} -include-managed-fields
```

## Acknowledgment

- [ratatui](https://github.com/ratatui-org/ratatui)
- [difftastic](https://github.com/Wilfred/difftastic)
- [kube-rs](https://github.com/kube-rs/kube-rs)
- [rust](https://github.com/rust-lang/rust)
