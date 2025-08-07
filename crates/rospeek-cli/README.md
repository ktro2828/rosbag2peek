# rospeek-cli

A crate for the command line interface for `rospeek`.

## Usage

```shell
cargo install --path crates/rospeek-cli
```

### List Topics

```shell
rospeek list <BAG_FILE>
```

```shell
  Topic name  |  [Message type]  |  (Serialization format)
- /test_topic [std_msgs/String] (cdr)
```

### Show a Specific Topic

```shell
rospeek show <BAG_FILE> --topic <TOPIC_NAME> --count <NUM_COUNT>
```

```shell
[0] t = 1234567890 ns, 7 bytes
```

### Decode CDR-encoded messages into JSON

```shell
rosppek decode <BAG_FILE> --topic <TOPIC_NAME>
```

```shell
=== Message 0 ===
Object {
    "data": String("hello"),
}
```
