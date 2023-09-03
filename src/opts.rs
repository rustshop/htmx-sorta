use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    #[arg(long, short, default_value = "[::1]:3000")]
    pub listen: String,

    #[arg(long, default_value = "db.redb")]
    pub db: PathBuf,
}
