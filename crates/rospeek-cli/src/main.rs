use clap::{Parser, Subcommand};
use rospeek_core::{BagReader, CdrDecoder, MessageSchema, RosPeekError, RosPeekResult};
use rospeek_db3::Db3Reader;
use std::{collections::BTreeMap, fs::File, path::PathBuf};

#[derive(Parser)]
#[command(name = "rospeek", about = "Peek into rosbag files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show bag file information and list all topics in the bag file
    Info {
        #[arg(value_name = "BAGFILE", help = "Path to the .db3 bag file")]
        bag: PathBuf,
    },

    /// Show the first N messages of a topic
    Show {
        #[arg(value_name = "BAGFILE", help = "Path to the .db3 bag file")]
        bag: PathBuf,

        #[arg(short, long, help = "Topic name to read messages (e.g. /tf)")]
        topic: String,

        #[arg(short, long, help = "Number of messages to show")]
        count: Option<usize>,
    },

    /// Decode CDR-encoded messages into JSON
    Decode {
        #[arg(value_name = "BAGFILE", help = "Path to the .db3 bag file")]
        bag: PathBuf,

        #[arg(short, long, help = "Topic name to decode (e.g. /tf)")]
        topic: String,
    },
}

fn main() -> RosPeekResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { bag } => {
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let stats = reader.stats();

            println!("File:             {}", stats.path);
            println!("Bag size:         {:.3} GiB", stats.size_bytes);
            println!("Storage type:     {}", stats.storage_type);
            println!("Duration:         {} s", stats.duration_sec);
            println!("Start:            {}", stats.start_time);
            println!("End:              {}", stats.end_time);
            println!("Topic Information:");

            // group topics by namespace
            let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for topic in reader.topics()? {
                let key = topic
                    .name
                    .split("/")
                    .filter(|s| !s.is_empty())
                    .next()
                    .map(|s| format!("/{}", s))
                    .unwrap_or_else(|| "/".to_string());
                grouped.entry(key).or_default().push(topic);
            }
            for (_, mut topics) in grouped {
                // sort topics by topic name
                topics.sort_by(|a, b| a.name.cmp(&b.name));
                for topic in topics {
                    println!(
                        "   - Topic: {} | Type: {} | Count: {} | Serialization Format: {}",
                        topic.name, topic.type_name, topic.count, topic.serialization_format
                    );
                }
            }
        }
        Commands::Show { bag, topic, count } => {
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let messages = reader.read_messages(&topic)?;
            let n = count.unwrap_or(messages.len());
            for (i, msg) in messages.iter().take(n).enumerate() {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len());
            }
        }
        Commands::Decode { bag, topic } => {
            println!(">> Start decoding: {}", topic);
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let topic_info = reader
                .topics()?
                .into_iter()
                .find(|t| t.name == topic)
                .ok_or_else(|| RosPeekError::TopicNotFound(topic.clone()))?;

            let schema = MessageSchema::try_from(topic_info.type_name.as_ref())?;
            let mut decoder = CdrDecoder::from_schema(&schema);

            let messages = reader.read_messages(&topic)?;
            let mut results = Vec::new();
            for (_, msg) in messages.iter().enumerate() {
                let value = decoder.reset(&msg.data).decode(&schema)?;
                results.push(value);
            }
            println!("✨Finish decoding all messages");
            println!(">> Start saving results as JSON");
            let filename = topic.trim_start_matches('/').replace('/', ".") + ".json";
            let writer = File::create(&filename)?;
            serde_json::to_writer_pretty(writer, &results)
                .map_err(|_| RosPeekError::Other("Failed to write JSON".to_string()))?;
            println!("✨Success to save JSON to: {}", filename);
        }
    }

    Ok(())
}
