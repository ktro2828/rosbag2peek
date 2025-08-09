# rospeek

`rospeek` is a blazing-fast, cross-platform ROS 2 bag analyzer written in Rust.

## Features

- Cross-platform -- Works on Linux, macOS, and Windows
- Format support -- ROS 2 `.db3`

## Quick Start

### Build

```shell
cargo build --release
```

### Install CLI

```shell
cargo install --path crates/rospeek-cli
```

Then, you can refer to usage of CLI with `rospeek -h`.

#### Show Bag Information and List Topics

```shell
rospeek info <BAG_FILE>
```

#### Show Topic Messages

```shell
rospeek show <BAG_FILE> -t <TOPIC_NAME>
```

#### Decode Topic Messages into JSON

```shell
rospeek decode <BAG_FILE> -t <TOPIC_NAME>
```
