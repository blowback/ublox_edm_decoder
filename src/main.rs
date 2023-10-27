use anyhow::Result;
use clap::Parser;
use hex;

use nom::Err::{Error, Failure, Incomplete};
use nom::{Err, IResult};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

use ublox_edm_decoder::edm::parser::parse_edm;
use ublox_edm_decoder::edm::subframe::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: std::path::PathBuf,

    #[arg(short, long)]
    collect_path: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut raw = fs::read_to_string(args.path)?;

    remove_whitespace(&mut raw);

    let bytes = hex::decode(raw)?;
    // println!("bytes: {:02x?}", bytes);
    let mut bs = bytes.as_slice();

    let mut bufs: Vec<&[u8]> = Vec::new();

    loop {
        let res = parse_edm(bs);

        match res {
            Ok((xbs, pkt)) => {
                bs = xbs;

                let frame = pkt.unwrap();
                println!("{}", frame);

                match frame.subframe {
                    EDMSubframe::DataEvent(data_event) => {
                        if args.collect_path.is_some() {
                            bufs.push(data_event.payload);
                        }
                    }
                    _ => {}
                }
            }

            Err(Error(e)) => {
                bs = &bs[1..];
            }

            Err(Failure(e)) => {
                break;
            }

            Err(Incomplete(e)) => {
                break;
            }
        };
    }

    if let Some(path) = args.collect_path {
        let f = File::create(path)?;

        let mut writer = BufWriter::new(f);
        println!("BUFS: {:?}", bufs);

        for b in bufs.iter() {
            println!("BUF: {:?}", b);
            writer.write(b).expect("can't write to file");
        }
        writer.flush();
    }

    Ok(())
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}
