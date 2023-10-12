use std::{error::Error, sync::mpsc::Receiver};

use midir::MidiOutputConnection;

use crate::midi_thread::MidiPayload;

pub enum Request<Addr> {
    // Sender
    OpenRoom,     // block on listening for invitation
    AcceptClient, // initiator_token / accept invitation and forward all midi received

    // Receiver
    JoinRoom(Addr), // send invitation to specified address:port
}

#[derive(Debug)]
pub enum Response<Addr> {
    // Sender
    NewClient(Addr),
    HasHangUp,

    // Receiver
    StartReceiving,

    // Error
    Err(String),
}

type Responder<Addr> = oneshot::Sender<Response<Addr>>;

pub trait Messenger {
    type Addr;
    fn new_sender(rx: Receiver<MidiPayload>, addr: Self::Addr) -> Self;
    fn new_receiver(midi_out: MidiOutputConnection) -> Self;
    fn req(&self, req: Request<Self::Addr>) -> Result<Response<Self::Addr>, Box<dyn Error>>;
    fn info(&self) -> String;
}

#[derive(Debug)]
pub struct midi_frame {
    len: usize,
    payload: [u8; 32],
}

impl From<&[u8]> for midi_frame {
    fn from(value: &[u8]) -> Self {
        let mut buf: [u8; 32] = [0; 32];
        for (b, m) in buf.iter_mut().zip(value) {
            *b = *m;
        }
        midi_frame {
            len: value.len(),
            payload: buf,
        }
    }
}

impl midi_frame {
    fn get_payload(src: &[u8; 33]) -> &[u8] {
        &src[1..(src[0] as usize + 1)]
    }
    fn serialize(&self) -> [u8; 33] {
        let mut whole: [u8; 33] = [0; 33];
        let (one, two) = whole.split_at_mut(1);
        one.copy_from_slice(&[self.len as u8]);
        two.copy_from_slice(&self.payload);
        whole
    }
}

pub mod tcp_messenger;
// pub mod bluetooth_messenger;
