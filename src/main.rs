use anyhow::Result;
use clap::Parser;
use colored::*;
use hex;
use thiserror::Error;

use nom::bytes::complete::{tag, take};
use nom::combinator::{map, peek};
use nom::multi::length_data;
use nom::number::complete::be_u16;
use nom::sequence::tuple;
use nom::Err::{Error, Failure, Incomplete};
use nom::{Err, IResult};

use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

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
    // println!("input: {}", raw);
    remove_whitespace(&mut raw);

    // for line in raw.lines() {
    // let bytes = hex::decode(line);
    // println!("bytes: {:?}", bytes);
    // }
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
                let c: IResult<&[u8], &[u8]> = take(1u32)(e.input);

                match c {
                    Ok((xbs, _)) => {
                        bs = xbs;
                    }
                    _ => {
                        break;
                    }
                }
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
    #[error("bad frame data")]
    BadFrameData,
}

#[derive(Debug)]
pub struct EDMFrame<'a> {
    id: EDMIdentifier,
    ftype: EDMType,
    subframe: EDMSubframe<'a>,
}

#[derive(Debug)]
pub enum EDMSubframe<'a> {
    ConnectEvent(EDMConnectEvent<'a>),
    DisconnectEvent(EDMDisconnectEvent),
    DataEvent(EDMDataEvent<'a>),
    DataCommand(EDMDataCommand<'a>),
    AtRequest(EDMAtRequest<'a>),
    AtResponse(EDMAtResponse<'a>),
    AtEvent(EDMAtEvent<'a>),
    ResendConnectEventsCommand(EDMResendConnectEventsCommand),
    IphoneEvent(EDMIphoneEvent),
    StartEvent(EDMStartEvent),
}

impl<'a> fmt::Display for EDMSubframe<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EDMSubframe::ConnectEvent(x) => write!(f, "{} {}", "ConnectEvt".cyan(), x),
            EDMSubframe::DisconnectEvent(x) => write!(f, "{} {}", "DisconnectEvt".cyan(), x),
            EDMSubframe::DataEvent(x) => write!(f, "{} {}", "DataEvt".cyan(), x),
            EDMSubframe::DataCommand(x) => write!(f, "{} {}", "DataCmd".cyan(), x),
            EDMSubframe::AtRequest(x) => write!(f, "{} {}", "ATReq".cyan(), x),
            EDMSubframe::AtResponse(x) => write!(f, "{} {}", "ATRes".cyan(), x),
            EDMSubframe::AtEvent(x) => write!(f, "{} {}", "ATEvt".cyan(), x),
            EDMSubframe::ResendConnectEventsCommand(x) => {
                write!(f, "{} {}", "ResendCECmd".cyan(), x)
            }
            EDMSubframe::IphoneEvent(x) => write!(f, "{} {}", "IphoneEvt".cyan(), x),
            EDMSubframe::StartEvent(x) => write!(f, "{} {}", "StartEvt".cyan(), x),
            _ => write!(f, "unknown"),
        }
    }
}

#[derive(Error, Debug)]
pub enum EDMConnectTypeError {
    #[error("bad connect type")]
    BadConnectType,
}

#[derive(Debug)]
pub enum EDMConnectType {
    Bluetooth = 1,
    IPv4,
    IPv6,
}

impl TryFrom<u8> for EDMConnectType {
    type Error = EDMConnectTypeError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == EDMConnectType::Bluetooth as u8 => Ok(EDMConnectType::Bluetooth),
            x if x == EDMConnectType::IPv4 as u8 => Ok(EDMConnectType::IPv4),
            x if x == EDMConnectType::IPv6 as u8 => Ok(EDMConnectType::IPv6),
            _ => Err(EDMConnectTypeError::BadConnectType),
        }
    }
}

impl fmt::Display for EDMConnectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EDMConnectType::Bluetooth => "BLUETOOTH".blue(),
                EDMConnectType::IPv4 => "IPv4".blue(),
                EDMConnectType::IPv6 => "IPv6".blue(),
            }
        )
    }
}

#[derive(Debug)]
pub struct EDMConnectEvent<'a> {
    channel_id: u8,
    connect_type: EDMConnectType,
    payload: &'a [u8],
}

impl<'a> EDMConnectEvent<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, EDMConnectTypeError> {
        let channel_id = bytes[0];
        let connect_type = EDMConnectType::try_from(bytes[1])?;

        Ok(Self {
            channel_id,
            connect_type,
            payload: &bytes[2..],
        })
    }
}

impl<'a> fmt::Display for EDMConnectEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "channel id: {:#02x}, connect type: {}, payload: {}",
            self.channel_id,
            self.connect_type,
            hex::encode(self.payload).red()
        )
    }
}

#[derive(Debug)]
pub struct EDMDisconnectEvent {
    channel_id: u8,
}

impl EDMDisconnectEvent {
    pub fn new(bytes: &[u8]) -> Result<Self, ()> {
        Ok(Self {
            channel_id: bytes[0],
        })
    }
}

impl fmt::Display for EDMDisconnectEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel id: {:#02x}", self.channel_id)
    }
}

#[derive(Debug)]
pub struct EDMDataEvent<'a> {
    channel_id: u8,
    payload: &'a [u8],
}

impl<'a> EDMDataEvent<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
        Ok(Self {
            channel_id: bytes[0],
            payload: &bytes[1..],
        })
    }
}

impl<'a> fmt::Display for EDMDataEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "channel id: {:#02x}, payload: {}",
            self.channel_id,
            hex::encode(self.payload).green()
        )
    }
}

#[derive(Debug)]
pub struct EDMDataCommand<'a> {
    channel_id: u8,
    payload: &'a [u8],
}

impl<'a> EDMDataCommand<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
        Ok(Self {
            channel_id: bytes[0],
            payload: &bytes[1..],
        })
    }
}

impl<'a> fmt::Display for EDMDataCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "channel id: {:#02x}, payload: {}",
            self.channel_id,
            hex::encode(self.payload).green()
        )
    }
}

#[derive(Debug)]
pub struct EDMAtRequest<'a> {
    payload: &'a [u8],
}

impl<'a> EDMAtRequest<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
        Ok(Self { payload: bytes })
    }
}

impl<'a> fmt::Display for EDMAtRequest<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txt = String::from_utf8_lossy(self.payload).into_owned();
        txt.retain(|c| c != '\n' && c != '\r');
        write!(f, "=> {}", txt.yellow())
    }
}

#[derive(Debug)]
pub struct EDMAtResponse<'a> {
    payload: &'a [u8],
}

impl<'a> EDMAtResponse<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
        Ok(Self { payload: bytes })
    }
}

impl<'a> fmt::Display for EDMAtResponse<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txt = String::from_utf8_lossy(self.payload).into_owned();
        txt.retain(|c| c != '\n' && c != '\r');
        write!(f, "=> {}", txt.yellow())
    }
}

#[derive(Debug)]
pub struct EDMAtEvent<'a> {
    payload: &'a [u8],
}

impl<'a> EDMAtEvent<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, ()> {
        Ok(Self { payload: bytes })
    }
}

impl<'a> fmt::Display for EDMAtEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut txt = String::from_utf8_lossy(self.payload).into_owned();
        txt.retain(|c| c != '\n' && c != '\r');
        write!(f, "=> {}", txt.yellow())
    }
}

#[derive(Debug)]
pub struct EDMResendConnectEventsCommand {}

impl EDMResendConnectEventsCommand {
    pub fn new() -> Self {
        Self {}
    }
}

impl fmt::Display for EDMResendConnectEventsCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[no data]")
    }
}

#[derive(Debug)]
pub struct EDMIphoneEvent {}

impl EDMIphoneEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl fmt::Display for EDMIphoneEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[no data]")
    }
}

#[derive(Debug)]
pub struct EDMStartEvent {}

impl EDMStartEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl fmt::Display for EDMStartEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[no data]")
    }
}

impl<'a> EDMFrame<'a> {
    pub fn new(id: EDMIdentifier, ftype: EDMType, subframe: EDMSubframe<'a>) -> Self {
        Self {
            id,
            ftype,
            subframe,
        }
    }

    pub fn from_parts(id: u16, ftype: u16, payload: &'a [u8]) -> Result<Self, EDMFrameError> {
        let xid = EDMIdentifier::try_from(id).map_err(|_| EDMFrameError::BadFrameIdentifier)?;
        let xftype = EDMType::try_from(ftype).map_err(|_| EDMFrameError::BadFrameType)?;

        let subframe = match (&xid, &xftype) {
            (EDMIdentifier::Connect, EDMType::Event) => EDMSubframe::ConnectEvent(
                EDMConnectEvent::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::Disconnect, EDMType::Event) => EDMSubframe::DisconnectEvent(
                EDMDisconnectEvent::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::Data, EDMType::Event) => EDMSubframe::DataEvent(
                EDMDataEvent::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::Data, EDMType::Command) => EDMSubframe::DataCommand(
                EDMDataCommand::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::AT, EDMType::Request) => EDMSubframe::AtRequest(
                EDMAtRequest::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::AT, EDMType::Response) => EDMSubframe::AtResponse(
                EDMAtResponse::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::AT, EDMType::Event) => EDMSubframe::AtEvent(
                EDMAtEvent::new(payload).map_err(|_| EDMFrameError::BadFrameData)?,
            ),
            (EDMIdentifier::ResendConnectEvents, EDMType::Command) => {
                EDMSubframe::ResendConnectEventsCommand(EDMResendConnectEventsCommand::new())
            }
            (EDMIdentifier::Iphone, EDMType::Event) => {
                EDMSubframe::IphoneEvent(EDMIphoneEvent::new())
            }
            (EDMIdentifier::Start, EDMType::Event) => EDMSubframe::StartEvent(EDMStartEvent::new()),

            (_, _) => panic!("arse"),
        };

        Ok(Self {
            id: xid,
            ftype: xftype,
            subframe,
        })
    }
}

impl<'a> fmt::Display for EDMFrame<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.subframe)
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
            tag([END])
    ))(input)?;

    let (payload, (id, typ)) = map(be_u16, |x| (x >> 4, x & 7))(t.1 .1)?;
    let frame = EDMFrame::from_parts(id, typ, payload);
    Ok((t.0, frame))
}

// fn scan_edm(input: &[u8]) -> IResult<&[u8]
