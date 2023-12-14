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
    len_le_bytes: [u8; 8],
    content: Option<Vec<u8>>,
    content_parsed: usize,
}

impl From<&[u8]> for MidiFrame {
    fn from(value: &[u8]) -> Self {
        MidiFrame {
            len: value.len(),
            len_le_bytes: [0; 8],
            content: Some(value.to_vec()),
            content_parsed: value.len(),
        }
    }
}

impl MidiFrame {
    #[doc(hidden)]
    /// Return Midi Message
    pub fn deser(mut self, src: &[u8]) -> Option<Vec<u8>> {
        for chunk in src.chunks(8) {
            match self.content_parsed {
                0..=7 => {
                    for (i, x) in chunk.iter().enumerate() {
                        self.len_le_bytes[self.content_parsed + i] = *x;
                    }
                }
                _ => self.content.as_mut().unwrap().extend_from_slice(chunk),
            }
            // self.content_parsed += chunk.len();
            if self.len_le_bytes.len() == 8 {
                self.len = usize::from_ne_bytes(self.len_le_bytes.into());
                self.content = Some(Vec::with_capacity(self.len));
            }
        }

        if self
            .content
            .as_ref()
            .is_some_and(|content| content.len() == self.len)
        {
            return self.content.take();
        }

        None
    }

    /// Return Network Message
    pub fn ser(mut self) -> Option<Vec<u8>> {
        let content = self.content.take()?;

        let mut res = Vec::with_capacity(self.len + 8);
        res.extend(self.len.to_ne_bytes().iter());
        res.extend(content);

        Some(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let src: [u8; 8] = [123, 1, 2, 3, 4, 5, 6, 244];
        let src_midi_frame: MidiFrame = MidiFrame::from(&src[..2]);

        println!("{:?}", src_midi_frame);
        println!("{:?}", src_midi_frame.ser())
    }
}
