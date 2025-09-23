use clap::{Parser, Subcommand, ValueEnum};
use rospeek_core::{RosPeekError, RosPeekResult, flatten_json, try_decode_json};
use rospeek_gui::{create_reader, spawn_app};
use serde_json::Value;
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
            for (i, msg) in messages.iter().take(n).enumerate() {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len());
            }
        }
        Commands::Dump { bag, topic, format } => {
            println!(">> Start decoding: {}", topic);
            let reader = create_reader(bag)?;
            let results = try_decode_json(reader, &topic)?;
            println!("✨Finish decoding all messages");
            println!(">> Start dumping results into {:?}", format);
            match format {
                Format::Json => {
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".json";
                    let writer = File::create(&filename)?;
                    serde_json::to_writer_pretty(writer, &results)
                        .map_err(|_| RosPeekError::Other("Failed to write JSON".to_string()))?;
                    println!("✨Success to save JSON to: {}", filename);
                }
                Format::Csv => {
                    println!(">> Start dumping results into CSV");
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".csv";
                    let writer = File::create(&filename)?;
                    let mut csv_writer = csv::WriterBuilder::new().from_writer(writer);
                    for (i, value) in results.iter().enumerate() {
                        if let Value::Object(object) = value {
                            let row = flatten_json(object)?;
                            if i == 0 {
                                let columns = row.keys().collect::<Vec<_>>();
                                csv_writer.write_record(columns).map_err(|e| {
                                    RosPeekError::Other(format!(
                                        "Failed to write CSV header: {}",
                                        e
                                    ))
                                })?;
                            }

                            let value_strings = row
                                .values()
                                .map(|value| serde_json::to_string(value).unwrap())
                                .collect::<Vec<_>>();

                            csv_writer.write_record(value_strings).map_err(|e| {
                                RosPeekError::Other(format!("Failed to write CSV row: {}", e))
                            })?;
                        }
                    }
                    println!("✨Success to save CSV to: {}", filename);
                }
            }
        }
        Commands::App => spawn_app().map_err(|e| RosPeekError::Other(format!("{e}")))?,
    }

    Ok(())
}
