//
//	MIDI PARSER
//

/// Struct used to parse Midi Message from incomming network messages
/// It keep track of the current unfinished message
#[derive(Debug, PartialEq, Eq)]
pub struct MidiParser {
    buffer: Option<Vec<u8>>,
}

impl MidiParser {
    /// Create a new MidiParser
    pub fn new() -> Self {
        MidiParser { buffer: None }
    }

    /// Parse all possible midi message in the given slice
    pub fn parse(&mut self, src: &[u8]) -> Vec<Vec<u8>> {
        let mut res: Vec<Vec<u8>> = vec![];

        for elem in src {
            if elem & 0x80 != 0 && *elem != 0xf7 {
                if let Some(buff) = self.buffer.take() {
                    res.push(buff);
                }
            }
            match self.buffer.as_mut() {
                Some(buf) => buf.push(*elem),
                None => self.buffer = Some(vec![*elem]),
            }
        }
        res
    }

    /// Return the cached unfinished midi message
    pub fn flush(&mut self) -> Option<Vec<u8>> {
        self.buffer.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_midi_messages() {
            let midi_messages: Vec<u8> = vec![
                0x90, 0x3C, 0x40, // Note On, Middle C, Velocity 64
                0x80, 0x3C, 0x40, // Note Off, Middle C, Velocity 64
                0xB0, 0x07, 0x7F, // Control Change, Volume, Max
                0xF0, // SysEx start
                0x43, // Manufacturer ID (Yamaha)
                0x10, // Device ID
                0x3E, // Model ID
                0x12, // Command ID
                0x00, 0x7F, 0x00, // Parameters
                0xF7, // SysEx end
            ];

            let expected: Vec<Vec<u8>> = vec![
                vec![0x90, 0x3C, 0x40],
                vec![0x80, 0x3C, 0x40],
                vec![0xB0, 0x07, 0x7F],
                vec![0xF0, 0x43, 0x10, 0x3E, 0x12, 0x00, 0x7F, 0x00, 0xF7],
            ];

            for chunk_size in 1..midi_messages.len() {
                let mut midi_parser = MidiParser::new();
                let mut out: Vec<Vec<u8>> = vec![];
                for msg in midi_messages.chunks(chunk_size) {
                    out.append(&mut midi_parser.parse(msg));
                }
                if let Some(res) = midi_parser.flush() {
                    out.push(res);
                }
                assert_eq!(out, expected);
                println!("{out:x?}");
            }
        }
    }
}
