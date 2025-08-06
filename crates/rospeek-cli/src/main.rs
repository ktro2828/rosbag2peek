use clap::{Parser, Subcommand};
use rospeek_core::{BagReader, RosbagResult};
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
    }

    Ok(())
}
