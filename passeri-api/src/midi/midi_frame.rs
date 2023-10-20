//
//	MIDI FRAME OVER NETWORK
//

/// Struct used to serialize and deserialize MIDI messages through network
///
/// the frame is composed of Nth bytes:
/// - Byte 0: length of the MIDI message
/// - Byte 1..N: raw MIDI message
///
/// This is subject to change, it is still on active development
#[derive(Debug)]
pub struct MidiFrame {
    len: usize,
    payload: [u8; 32],
}

impl From<&[u8]> for MidiFrame {
    fn from(value: &[u8]) -> Self {
        let mut buf: [u8; 32] = [0; 32];
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
    pub fn get_payload(src: &[u8; 33]) -> &[u8] {
        &src[1..(src[0] as usize + 1)]
    }
    #[doc(hidden)]
    pub fn serialize(&self) -> [u8; 33] {
        let mut whole: [u8; 33] = [0; 33];
        let (one, two) = whole.split_at_mut(1);
        one.copy_from_slice(&[self.len as u8]);
        two.copy_from_slice(&self.payload);
        whole
    }
}
