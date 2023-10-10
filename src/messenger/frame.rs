use std::{io::Cursor, u8};
use byteorder::{BigEndian, ReadBytesExt}; // 1.2.7


use bytes::BytesMut;

#[derive(Debug)]
pub enum Frame{
	IN {
		initiator_token: u32,
		ssrc: u32,
		name: String,
	},
	OK {
		initiator_token: u32,
		ssrc: u32,
		name: String
	},
	NO {
		initiator_token: u32,
		ssrc: u32,
	}
}

pub enum Error {
	Incomplete,
	Invalid
}

impl Frame {
    /// The message has already been validated with `check`.
    pub fn parse(src: &BytesMut) -> Result<Frame, Error> {
		if src.len() < 32 * 4 {
			return Err(Error::Incomplete)
		}

		if src[0] != 0xFF && src[1] != 0xFF && get_u32(&src[4..8])? != 2 {
			return Err(Error::Invalid)
		}

		match src[2..4] {
			[0x49, 0x4E] => Ok(Frame::IN { initiator_token: get_u32(&src[8..12])?, ssrc: get_u32(&src[12..16])?, name: get_name(src)? }),
			[0x4F, 0x4B] => Ok(Frame::OK { initiator_token: get_u32(&src[8..12])?, ssrc: get_u32(&src[12..16])?, name: get_name(src)? }),
			[0x4E, 0x4F] => Ok(Frame::NO { initiator_token: get_u32(&src[8..12])?, ssrc: get_u32(&src[12..16])? }),
			_ => Err(Error::Invalid)
		}
    }

}

fn get_name(src: &BytesMut) -> Result<String, Error> {
	for letter in &src[16..] {
		println!("{}", letter);
	}
	Ok("bonjour".into())
}

fn get_u32(src: &[u8]) -> Result<u32, Error> {
	Ok(src[0..4]
		.iter()
		.fold(0, |acc, x| acc << 8 + x))
}
