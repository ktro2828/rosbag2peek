use clap::{Parser, Subcommand, ValueEnum};
use rospeek_core::{RosPeekError, RosPeekResult, try_decode_csv, try_decode_json};
use rospeek_gui::{create_reader, spawn_app};
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
        #[arg(value_name = "BAGFILE", help = "Path to the [.db3, .mcap] bag file")]
        bag: PathBuf,
    },

    /// Show the first N messages of a topic
    Show {
        #[arg(value_name = "BAGFILE", help = "Path to the [.db3, .mcap] bag file")]
        bag: PathBuf,

        #[arg(short, long, help = "Topic name to read messages (e.g. /tf)")]
        topic: String,

        #[arg(short, long, help = "Number of messages to show")]
        count: Option<usize>,
    },

    /// Decode CDR-encoded messages and dump them into JSON
    Dump {
        #[arg(value_name = "BAGFILE", help = "Path to the [.db3, .mcap] bag file")]
        bag: PathBuf,

        #[arg(short, long, help = "Topic name to decode (e.g. /tf)")]
        topic: String,

        #[arg(
            short,
            long,
            value_enum,
            default_value = "json",
            help = "Output format"
        )]
        format: Format,

        #[arg(long, help = "Timestamp in nanoseconds since which to read messages")]
        since: Option<u64>,

        #[arg(long, help = "Timestamp in nanoseconds until which to read messages")]
        until: Option<u64>,
    },

    /// Spawn GUI application
    App,
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Json,
    Csv,
}

fn main() -> RosPeekResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { bag } => {
            let reader = create_reader(bag)?;

            println!("{}", reader.stats());

            println!("Topic Information:");
            // group topics by namespace
            let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for topic in reader.topics()? {
                let key = topic
                    .name
                    .split("/")
                    .find(|s| !s.is_empty())
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
            let reader = create_reader(bag)?;

            let messages = reader.read_messages(&topic)?;
            let n = count.unwrap_or(messages.len());
            messages.iter().take(n).enumerate().for_each(|(i, msg)| {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len())
            });
        }
        Commands::Dump {
            bag,
            topic,
            format,
            since,
            until,
        } => {
            println!(">> Start decoding: {}", topic);
            let reader = create_reader(bag)?;
            println!("✨Finish decoding all messages");
            println!(">> Start dumping results into {:?}", format);
            let filename = match format {
                Format::Json => {
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".json";
                    let writer = File::create(&filename)?;
                    let values = try_decode_json(reader, &topic, since, until)?;
                    serde_json::to_writer_pretty(writer, &values)
                        .map_err(|_| RosPeekError::Other("Failed to write JSON".to_string()))?;
                    filename
                }
                Format::Csv => {
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".csv";
                    let writer = File::create(&filename)?;
                    let mut csv_writer = csv::WriterBuilder::new().from_writer(writer);
                    let (columns, values) = try_decode_csv(reader, &topic, since, until)?;
                    csv_writer.write_record(columns).map_err(|e| {
                        RosPeekError::Other(format!("Failed to write CSV header: {}", e))
                    })?;
                    for value in values {
                        csv_writer.write_record(value).map_err(|e| {
                            RosPeekError::Other(format!("Failed to write CSV row: {}", e))
                        })?
                    }
                    filename
                }
            };
            println!("✨Success to save {:?} to: {}", format, filename);
        }
        Commands::App => spawn_app().map_err(|e| RosPeekError::Other(format!("{e}")))?,
    }

    Ok(())
}
