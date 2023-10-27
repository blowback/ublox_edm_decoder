use crate::edm::subframe::*;
use crate::edm::types::*;
use std::fmt;

#[derive(Debug)]
pub struct EDMFrame<'a> {
    pub id: EDMIdentifier,
    pub ftype: EDMType,
    pub subframe: EDMSubframe<'a>,
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
