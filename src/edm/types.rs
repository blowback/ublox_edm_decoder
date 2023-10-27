use colored::*;
use std::fmt;

use thiserror::Error;

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
