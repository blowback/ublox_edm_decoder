use nom::bytes::complete::{tag, take};
use nom::combinator::{map, peek};
use nom::multi::length_data;
use nom::number::complete::be_u16;
use nom::sequence::tuple;

use crate::edm::frame::*;
use crate::edm::types::*;
use nom::{Err, IResult};

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
pub fn parse_edm(input: &[u8]) -> IResult<&[u8], Result<EDMFrame, EDMFrameError>> {
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
