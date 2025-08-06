use clap::{Parser, Subcommand};
use rospeek_core::{BagReader, CdrDecoder, MessageSchema, RosbagError, RosbagResult};
use rospeek_db3::Db3Reader;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rospeek", about = "Peek into rosbag files", long_about = None)]
struct Cli {
    #[arg(short, long)]
    bag: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all topics int the bag
    ListTopics,

    /// Show the first N messages of a topic
    Show {
        #[arg(short, long)]
        topic: String,

        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },

    /// Decode CDR to json
    Decode {
        #[arg(short, long)]
        topic: String,
    },
}

fn main() -> RosbagResult<()> {
    let cli = Cli::parse();

    let reader = Db3Reader::open(cli.bag)?;

    match cli.command {
        Commands::ListTopics => {
            let topics = reader.topics()?;
            for topic in topics {
                println!(
                    "- {} [{}] ({})",
                    topic.name, topic.type_name, topic.serialization_format
                );
            }
        }
        Commands::Show { topic, count } => {
            let messages = reader.read_messages(&topic)?;
            for (i, msg) in messages.iter().take(count).enumerate() {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len());
            }
        }
        Commands::Decode { topic } => {
            let topic_info = reader
                .topics()?
                .into_iter()
                .find(|t| t.name == topic)
                .ok_or_else(|| RosbagError::Other(format!("Topic not found: {topic}")))?;

            let schema = MessageSchema::try_from(topic_info.type_name.as_ref()).map_err(|_| {
                RosbagError::Other(format!("Could not load IDL for {}", topic_info.type_name))
            })?;

            let messages = reader.read_messages(&topic)?;

            for (i, msg) in messages.iter().enumerate() {
                println!("=== Message {} ===", i);
                let mut decoder = CdrDecoder::new(&msg.data);
                let value = decoder
                    .decode(&schema)
                    .map_err(|_| RosbagError::Other(format!("Failed to decode")))?;

                println!("{:#?}", value);
            }
        }
    }

    Ok(())
}
