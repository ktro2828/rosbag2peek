use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

/// Output file format for the dump command.
#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum DumpFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

#[derive(Subcommand)]
pub(crate) enum Command {
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

        #[arg(long, help = "Number of messages to skip after filtering")]
        offset: Option<usize>,
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
        format: DumpFormat,

        #[arg(long, help = "Timestamp in nanoseconds since which to read messages")]
        since: Option<u64>,

        #[arg(long, help = "Timestamp in nanoseconds until which to read messages")]
        until: Option<u64>,

        #[arg(long, help = "Maximum number of messages to dump")]
        limit: Option<usize>,

        #[arg(long, help = "Number of messages to skip after filtering")]
        offset: Option<usize>,
    },

    /// Spawn GUI application
    App,
}
