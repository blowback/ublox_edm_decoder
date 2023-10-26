use anyhow::Result;
use clap::Parser;
use hex;
use thiserror::Error;

use nom::bytes::complete::{tag, take, take_till};
use nom::combinator::{map, peek, value};
use nom::error::{make_error, Error};
use nom::multi::length_data;
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::tuple;
use nom::Err;
use nom::IResult;

use std::fs;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut raw = fs::read_to_string(args.path)?;
    println!("input: {}", raw);
    remove_whitespace(&mut raw);

    // for line in raw.lines() {
    // let bytes = hex::decode(line);
    // println!("bytes: {:?}", bytes);
    // }
    let bytes = hex::decode(raw)?;
    println!("bytes: {:02x?}", bytes);
    let bs = bytes.as_slice();

    let Ok((xbs, pkt)) = parse_edm(bs) else { todo!() };
    println!("frame: {:?}", pkt.unwrap());
    Ok(())
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

// fn scan_for_edm<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], Error<&'a [u8]>> {
//
//     let (candidate, _crap) = take_till(|c| c == START)(input)?;
//     let (pkt, _header) = be_u8(candidate)?;
//     let (pkt, flags_len) = be_u16(pkt)?;
//     let len = flags_len & 0xfff;
//     let (remainder, payload) = take(len)(pkt)?;
//     let (remainder, trailer) = be_u8(remainder)?;
//
//     if trailer == END {
//         Ok((remainder, payload))
//     } else {
//         Err(Err::Error(make_error(
//             candidate,
//             nom::error::ErrorKind::TagBits,
//         )))
//     }
// }

#[derive(Debug)]
pub enum EDMType {
    Event = 0x1,
    Indication,
    Response,
    Request,
    Confirmation,
    Command,
}

impl TryFrom<u16> for EDMType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == EDMType::Event as u16 => Ok(EDMType::Event),
            x if x == EDMType::Indication as u16 => Ok(EDMType::Indication),
            x if x == EDMType::Response as u16 => Ok(EDMType::Response),
            x if x == EDMType::Request as u16 => Ok(EDMType::Request),
            x if x == EDMType::Confirmation as u16 => Ok(EDMType::Confirmation),
            x if x == EDMType::Command as u16 => Ok(EDMType::Command),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum EDMIdentifier {
    Connect = 0x001,
    Disconnect,
    Data,
    AT,
    ResendConnectEvents,
    Iphone,
    Start,
}

impl TryFrom<u16> for EDMIdentifier {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == EDMIdentifier::Connect as u16 => Ok(EDMIdentifier::Connect),
            x if x == EDMIdentifier::Disconnect as u16 => Ok(EDMIdentifier::Disconnect),
            x if x == EDMIdentifier::Data as u16 => Ok(EDMIdentifier::Data),
            x if x == EDMIdentifier::AT as u16 => Ok(EDMIdentifier::AT),
            x if x == EDMIdentifier::ResendConnectEvents as u16 => {
                Ok(EDMIdentifier::ResendConnectEvents)
            }
            x if x == EDMIdentifier::Iphone as u16 => Ok(EDMIdentifier::Iphone),
            x if x == EDMIdentifier::Start as u16 => Ok(EDMIdentifier::Start),
            _ => Err(()),
        }
    }
}

#[derive(Error, Debug)]
pub enum EDMFrameError {
    #[error("bad frame identifier")]
    BadFrameIdentifier,
    #[error("bad frame type")]
    BadFrameType,
}

#[derive(Debug)]
pub struct EDMFrame<'a> {
    id: EDMIdentifier,
    ftype: EDMType,
    payload: &'a [u8],
}

impl<'a> EDMFrame<'a> {
    pub fn new(id: EDMIdentifier, ftype: EDMType, payload: &'a [u8]) -> Self {
        Self { id, ftype, payload }
    }

    pub fn from_parts(id: u16, ftype: u16, payload: &'a [u8]) -> Result<Self, EDMFrameError> {
        let id = EDMIdentifier::try_from(id).map_err(|_| EDMFrameError::BadFrameIdentifier)?;
        let ftype = EDMType::try_from(ftype).map_err(|_| EDMFrameError::BadFrameType)?;
        Ok(Self { id, ftype, payload })
    }
}

// EDM is 0xAA [flags_len] [payload] 0x55
// where flags_len is 4 reserved bits then 12 bits of payload length
// .. first stab
fn parse_edm1(input: &[u8]) -> IResult<&[u8], bool> {
    const START: u8 = 0xaa;
    const END: u8 = 0x55;
    let (p, _) = tag([START])(input)?;
    let (p, flags_len) = be_u16(p)?;
    let len = flags_len & 0xfff;
    let (p, payload) = take(len)(p)?;
    let (p, _) = peek(tag([END]))(p)?;
    Ok((p, true))
}

// ...combinate!
fn parse_edm(input: &[u8]) -> IResult<&[u8], Result<EDMFrame, EDMFrameError>> {
    const START: u8 = 0xaa;
    const END: u8 = 0x55;

    #[rustfmt::skip]
    let t = tuple((
            tag([START]), 
            length_data(map(be_u16, |x| x & 0xfff)),
            peek(tag([END]))
    ))(input)?;

    let (payload, (id, typ)) = map(be_u16, |x| (x >> 4, x & 7))(t.1 .1)?;
    let frame = EDMFrame::from_parts(id, typ, payload);
    Ok((t.0, frame))
}
