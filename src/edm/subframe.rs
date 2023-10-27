use colored::*;
use std::fmt;

use crate::edm::types::*;

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

#[derive(Debug)]
pub struct EDMConnectEvent<'a> {
    pub channel_id: u8,
    pub connect_type: EDMConnectType,
    pub payload: &'a [u8],
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
    pub channel_id: u8,
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
    pub channel_id: u8,
    pub payload: &'a [u8],
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
    pub channel_id: u8,
    pub payload: &'a [u8],
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
    pub payload: &'a [u8],
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
    pub payload: &'a [u8],
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
    pub payload: &'a [u8],
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
