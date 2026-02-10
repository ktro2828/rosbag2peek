# rospeek

<div align="center">
    <img src="./docs/assets/icon.png" alt="icon" width="40%"/>
</div>

`rospeek` is a blazing-fast, cross-platform ROS 2 bag analyzer written in Rust.

```txt
Execution time [s] (log scale)
0.1      1       10      100
|--------|-------|-------|

rospeek info   |█                               | 0.79 s
ros2 bag info  |██████████████████████████████  | 29.1 s  (~36.9× slower)
```

## Features

- Cross-platform -- Works on Linux, macOS, and Windows
- Format support -- ROS 2 `.db3`, `.mcap`

## Quick Start

### Build

```bash
git clone https://github.com/ktro2828/rosbag2peek.git
cd rosbag2peek
cargo build --release
```

### Install CLI

```bash
cargo install --path crates/rospeek-cli
```

Then, you can refer to usage of CLI with `rospeek -h`.

#### 1. Show Bag Information and List Topics

This command shows overviews of a bag file, similar to `rosbag2 info`:

```bash
rospeek info <BAG_FILE>
```

#### 2. List Topic Messages

This command shows a list of serialized messages:

```bash
rospeek show <BAG_FILE> -t <TOPIC_NAME>
```

#### 3. Decode Topic Messages and Dump into JSON/CSV

This command decodes topic messages and dumps them into JSON or CSV format.

```bash
rospeek dump <BAG_FILE> -t <TOPIC_NAME> [-f json|csv]
```

The output file is saved with the filename that separates the topic namespace by dots.

For example, the following command dumps `/foo/bar` into `foo.bar.json`:

```bash
rospeek dump <BAG_FILE> -t /foo/bar -f json
```

You can also dump messages between two timestamps:

```bash
rospeek dump <BAG_FILE> -t /foo/bar -f json --since 1640995200 --until 1640995260
```

#### 4. Spawn GUI

This command spawns a GUI application for visualizing bag files:

```bash
rospeek app
```
