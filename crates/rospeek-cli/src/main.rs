use clap::{Parser, Subcommand};
use rospeek_core::{BagReader, CdrDecoder, MessageSchema, RosPeekError, RosPeekResult};
use rospeek_db3::Db3Reader;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rospeek", about = "Peek into rosbag files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all topics int the bag
    ListTopics {
        #[arg(value_name = "BAGFILE", help = "Path to the .db3 bag file")]
        bag: PathBuf,
    },

    /// Show the first N messages of a topic
    Show {
        #[arg(value_name = "BAGFILE", help = "Path to the .db3 bag file")]
        bag: PathBuf,

        #[arg(short, long, help = "Topic name to read messages (e.g. /tf)")]
        topic: String,

        #[arg(short, long, default_value_t = 1, help = "Number of messages to show")]
        count: usize,
    },

    /// Decode CDR to json
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
        Commands::ListTopics { bag } => {
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let topics = reader.topics()?;
            for topic in topics {
                println!(
                    "- {} [{}] ({})",
                    topic.name, topic.type_name, topic.serialization_format
                );
            }
        }
        Commands::Show { bag, topic, count } => {
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let messages = reader.read_messages(&topic)?;
            for (i, msg) in messages.iter().take(count).enumerate() {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len());
            }
        }
        Commands::Decode { bag, topic } => {
            // TODO(ktro2828): add support of McapReader
            let reader: Box<dyn BagReader> = Box::new(Db3Reader::open(bag)?);

            let topic_info = reader
                .topics()?
                .into_iter()
                .find(|t| t.name == topic)
                .ok_or_else(|| RosPeekError::TopicNotFound(topic.clone()))?;

            let schema = MessageSchema::try_from(topic_info.type_name.as_ref())?;

            let messages = reader.read_messages(&topic)?;

            for (i, msg) in messages.iter().enumerate() {
                println!("=== Message {} ===", i);
                let value = CdrDecoder::new(&msg.data).decode(&schema)?;
                println!("{:#?}", value);
            }
        }
    }

    Ok(())
}
