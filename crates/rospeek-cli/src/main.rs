mod command;

use clap::Parser;
use rospeek_core::{RosPeekResult, try_decode_csv, try_decode_json};
use rospeek_gui::{create_reader, spawn_app};
use std::{collections::BTreeMap, fs::File};

use crate::command::{Command, DumpFormat};

#[derive(Parser)]
#[command(name = "rospeek", about = "Peek into rosbag files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> RosPeekResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Info { bag } => {
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
                    .map(|s| format!("/{s}"))
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
        Command::Show {
            bag,
            topic,
            count,
            offset,
        } => {
            let reader = create_reader(bag)?;

            let messages = reader.read_messages_range(&topic, None, None, count, offset)?;
            messages.iter().enumerate().for_each(|(i, msg)| {
                println!("[{}] t = {} ns, {} bytes", i, msg.timestamp, msg.data.len())
            });
        }
        Command::Dump {
            bag,
            topic,
            format,
            since,
            until,
            limit,
            offset,
        } => {
            println!(">> Start decoding: {topic}");
            let reader = create_reader(bag)?;
            println!("✨Successfully opened bag, starting to decode messages");
            println!(">> Start dumping results into {format:?}");
            let filename = match format {
                DumpFormat::Json => {
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".json";
                    let writer = File::create(&filename)?;
                    let values = try_decode_json(reader, &topic, since, until, limit, offset)?;
                    serde_json::to_writer_pretty(writer, &values)?;
                    filename
                }
                DumpFormat::Csv => {
                    let filename = topic.trim_start_matches('/').replace('/', ".") + ".csv";
                    let writer = File::create(&filename)?;
                    let mut csv_writer = csv::WriterBuilder::new().from_writer(writer);
                    let (columns, values) =
                        try_decode_csv(reader, &topic, since, until, limit, offset)?;
                    csv_writer.write_record(columns)?;
                    for value in values {
                        csv_writer.write_record(value)?
                    }
                    filename
                }
            };
            println!("✨Success to save {format:?} to: {filename}");
        }
        Command::App => spawn_app()?,
    }

    Ok(())
}
