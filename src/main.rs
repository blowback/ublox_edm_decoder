use anyhow::Result;
use clap::Parser;
use hex;
use std::fs;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let inp = fs::read_to_string(args.path)?;
    println!("input: {}", inp);

    for line in inp.lines() {
        let bytes = hex::decode(line);
        println!("bytes: {:?}", bytes);
    }
    Ok(())
}
