//
//	MIDI FRAME OVER NETWORK
//

const FRAME_SIZE: usize = 112;

/// Struct used to serialize and deserialize MIDI messages through network
///
/// the frame is composed of Nth bytes:
/// - Byte 0: length of the MIDI message
/// - Byte 1..N: raw MIDI message
///
/// This is subject to change, it is still on active development
#[derive(Debug, PartialEq, Eq)]
pub struct MidiFrame {
    len: usize,
    pub payload: [u8; FRAME_SIZE],
}

impl From<&[u8]> for MidiFrame {
    fn from(value: &[u8]) -> Self {
        let mut buf: [u8; FRAME_SIZE] = [0; FRAME_SIZE];
        for (b, m) in buf.iter_mut().zip(value) {
            *b = *m;
        }
        MidiFrame {
            len: value.len(),
            payload: buf,
        }
    }
}

impl MidiFrame {
    #[doc(hidden)]
    pub fn get_payload(src: &[u8; FRAME_SIZE + 1]) -> &[u8] {
        &src[1..(src[0] as usize + 1)]
    }
    #[doc(hidden)]
    pub fn serialize(&self) -> [u8; FRAME_SIZE + 1] {
        let mut whole: [u8; FRAME_SIZE + 1] = [0; FRAME_SIZE + 1];
        let (one, two) = whole.split_at_mut(1);
        one.copy_from_slice(&[self.len as u8]);
        two.copy_from_slice(&self.payload);
        whole
    }
}
