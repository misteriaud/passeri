//
//	MIDI FRAME OVER NETWORK
//

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
    pub fn get_payload(src: &[u8; 33]) -> &[u8] {
        &src[1..(src[0] as usize + 1)]
    }
    pub fn serialize(&self) -> [u8; 33] {
        let mut whole: [u8; 33] = [0; 33];
        let (one, two) = whole.split_at_mut(1);
        one.copy_from_slice(&[self.len as u8]);
        two.copy_from_slice(&self.payload);
        whole
    }
}
