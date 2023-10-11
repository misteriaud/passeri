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
	},
	BY {
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
    pub fn parse(src: &Vec<u8>) -> Result<Frame, Error> {
		if src.len() < 8 {
			return Err(Error::Incomplete)
		}

		if src.len() > 8 && (src[0] != 0xFF || src[1] != 0xFF || get_u32(&src[4..8]) != 2) {
			println!("{}", get_u32(&src[4..8]));
			return Err(Error::Invalid)
		}

		if src.len() < 4 * 4 {
			return Err(Error::Incomplete)
		}


		match src[2..4] {
			[0x49, 0x4E] => Ok(Frame::IN { initiator_token: get_u32(&src[8..12]), ssrc: get_u32(&src[12..16]), name: get_name(src)? }),
			[0x4F, 0x4B] => Ok(Frame::OK { initiator_token: get_u32(&src[8..12]), ssrc: get_u32(&src[12..16]), name: get_name(src)? }),
			[0x4E, 0x4F] => Ok(Frame::NO { initiator_token: get_u32(&src[8..12]), ssrc: get_u32(&src[12..16]) }),
			[0x42, 0x59] => Ok(Frame::BY { initiator_token: get_u32(&src[8..12]), ssrc: get_u32(&src[12..16]) }),
			_ => Err(Error::Invalid)
		}
    }

}

fn get_name(src: &[u8]) -> Result<String, Error> {
	let mut len: usize = 0;

	for (i, x) in src[16..].iter().enumerate() {
		if *x == 0 {
			len = i;
		}
	}
	match len {
		0 => Err(Error::Incomplete),
		len => Ok(String::from_utf8_lossy(&src[16..len]).into_owned())
	}
}

fn get_u32(src: &[u8]) -> u32 {
	src.iter()
		.fold(0, |acc, x| (acc << 8) + *x as u32)
}
