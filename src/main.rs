use std::time::Instant;

use clap::{Parser, Subcommand};

use baras::parse_log_file;

#[derive(Parser)]
#[command(version, about = "test")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// test command
    ParseFile {
        #[arg(short, long)]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ParseFile { path }) => {
            if !path.is_empty() {
                let timer = Instant::now();
                let data = parse_log_file(path).expect("failed to parse log file {path}");
                let ms = timer.elapsed().as_millis();
                println!("parsed {} events in {}ms", data.len(), ms);
            } else {
                println!("invalid path");
            }
        }
        None => {}
    }
}
