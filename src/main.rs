use std::path::Path;
use std::time::Duration;
mod app;
use app::App;
use clap::Parser;
mod server;
use clap::Subcommand;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = 3000)]
    port: u32,
    #[arg(long)]
    pdb: String,
    #[arg(short, long, default_value = "")]
    fallback: String,
    #[arg(short, long, default_value_t = 1)]
    replicas: usize,
    #[arg(short, long, default_value_t = 10)]
    subvolumes: usize,
    #[arg(short, long)]
    volumes: String,
    #[arg(long, default_value_t = false)]
    protect: bool,
    #[arg(long, default_value_t = true)]
    md5sum: bool,
    #[arg( long, default_value = "1",value_parser = parse_duration)]
    voltimeout: std::time::Duration,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Server,
    Rebuild,
    Rebalance,
}
fn parse_volumes(arg: &str) -> Result<Vec<String>, clap::Error> {
    let splitted = arg.split(",").map(|s| s.to_string()).collect();
    Ok(splitted)
}
fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(std::time::Duration::from_secs(seconds))
}
fn main() {
    let args = Args::parse();
    assert!(args.pdb.len() != 0, "Need a path to the db");
    assert!(
        args.volumes.len() >= args.replicas,
        "Need at least as many volumes as replicas"
    );
    let mut dbopts = leveldb::options::Options::new();
    dbopts.create_if_missing = true;
    let db: leveldb::database::Database<i32> =
        leveldb::database::Database::open(&Path::new(&args.pdb), dbopts).unwrap();
    let app = App {
        db,
        mlock: std::sync::Mutex::new(false),
        lock: std::collections::HashMap::new(),
        uploadids: std::collections::HashMap::new(),
        volumes: parse_volumes(&args.volumes).unwrap(),
        fallback: args.fallback,
        replicas: args.replicas,
        subvolumes: args.subvolumes,
        protect: args.protect,
        md5sum: args.md5sum,
        voltimeout: args.voltimeout,
    };

    match args.command {
        Commands::Server => {
            println!("Server");
        }
        Commands::Rebuild => {
            println!("Rebuild");
        }
        Commands::Rebalance => {
            println!("Rebalance");
        }
    }
}
