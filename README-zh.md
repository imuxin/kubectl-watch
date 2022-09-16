# kubectl-watch

[English](./README.md)

一个可以监听 kubernetes 资源的变更信息的 kubectl 插件。其中变更的内容通过使用 [delta](https://github.com/dandavison/delta) 或 [difftastic](https://github.com/Wilfred/difftastic) 工具提供漂亮的终端界面展示。

|                  使用 delta 概览                   |                使用 difftastic 概览                |
| :------------------------------------------------: | :------------------------------------------------: |
| ![overview-delta.png](./assets/overview-delta.png) | ![overview-difft.png](./assets/overview-difft.png) |


## 安装说明

### 方式一：使用 Docker 镜像 [推荐]

1. 您需要在环境里预先安装好 Docker，参考 [官网](https://docs.docker.com/engine/install/)；或者安装 containerd，参考 [安装教程](https://github.com/containerd/containerd/blob/main/docs/getting-started.md#installing-containerd) 和 [nerdctl](https://github.com/containerd/nerdctl) 命令行工具。
2. 拷贝 script 目录下的 kubectl-watch 脚本到环境的 $PATH 其中的一个目录下，比如 `/usr/local/bin`。
```bash
cp script/kubectl-watch /usr/local/bin/
chmod +x /usr/local/bin/kubectl-watch
```

### 从 [release assets](https://github.com/imuxin/kubectl-watch/releases) 下载可执行制品。
### 使用 [Cargo](https://crates.io/crates/kubectl-watch)进行源码编译安装。

```bash
cargo install kubectl-watch --locked
```

## Cmd 帮助

```bash
USAGE:
    kubectl-watch [OPTIONS] [ARGS]

ARGS:
    <RESOURCE>    Support resource 'plural', 'kind' and 'shortname'
    <NAME>        Resource name, optional

OPTIONS:
    -A, --all                       If present, list the requested object(s) across all namespaces
        --diff-tool <DIFF_TOOL>     Diff tool to analyze delta changes [default: delta] [possible values: delta, difft]
        --export <EXPORT>           A path, where all watched resources will be strored
    -h, --help                      Print help information
        --include-managed-fields    Set ture to show managed fields delta changes
    -l, --selector <SELECTOR>       Selector (label query) to filter on, supports '=', '==', and '!='.(e.g. -l key1=value1,key2=value2)
    -n, --namespace <NAMESPACE>     If present, the namespace scope for this CLI request
    -s, --skip-delta                Skip show delta changes view
        --use-tls                   Use tls to request api-server
    -V, --version                   Print version information
```

## 参考实例

监听所有命名空间下的 deployment 资源
```bash
kubectl-watch deployment -A
```

监听某个命名空间下的 depoyment 资源
```bash
kubectl-watch deployment -n {namespace}
```

监听某个命名空间下的某个 depoyment 资源
```bash
kubectl-watch deployment -n {namespace} {name}
```

追加 `--skip-delta` 选项，仅监听变动资源，同 `kubectl get -w`
```bash
kubectl-watch {resource} --delta
```

追加 `--diff-tool difft` 选项来使用 `difftastic` 工具显示变化内容
```bash
kubectl-watch {resource} --diff-tool difft
```

追加 `--export "/to/your/path"` 选项，导出监听的资源到本地存储
```bash
kubectl-watch {resource} --export "/to/your/path"
```

`managed-fields` 默认是不进行比对的, 追加 `--include-managed-fields` 选项，展示 managed fields 的变化
```bash
kubectl-watch {resource} -include-managed-fields
```

## 致谢

- [delta](https://github.com/dandavison/delta)
- [difftastic](https://github.com/Wilfred/difftastic)
- [kube-rs](https://github.com/kube-rs/kube-rs)
- [rust](https://github.com/rust-lang/rust)
