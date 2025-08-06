# rospeek-cli

A crate for the command line interface for `rospeek`.

## Usage

### List Topics

```shell
cargo run rospeek-cli -- -b <BAG_FILE.db3> list-topics
```

```shell
- /test_topic [std_msgs/String] (cdr)
```

### Show a Specific Topic

```shell
cargo run rospeek-cli -- -b <BAG_FILE.db3> show --topic <TOPIC_NAME> --count <NUM_COUNT>
```

```shell
[0] t = 1234567890 ns, 7 bytes
```
