use std::{sync::mpsc::Receiver, error::Error, net::SocketAddr};

use crate::{midi_thread::MidiPayload, rtp_midi};

pub enum Request<Addr> {
	// Receiver
	WaitForInvitation(Addr), // block on listening for invitation
	AcceptInvitation(rtp_midi::ControlPacket), // initiator_token / accept invitation and forward all midi received

	// Sender
	InviteSomeone(Addr), // send invitation to specified address:port
}

#[derive(Debug)]
pub enum Response {
	// Receiver
	InvitationReceived(rtp_midi::ControlPacket),
	StartReceiving,

	// Sender
	InvitationSended,
	StartSending,

	// Error
	Err
}

type Responder = oneshot::Sender<Response>;

pub trait Messenger {
	type Addr;
	fn new_sender(rx: Receiver<MidiPayload>) -> Self;
	fn req(&self, req: Request<Self::Addr>) -> Result<Response, Box<dyn Error>>;
	fn info(&self) -> String;
}

pub mod ip_messenger;
// pub mod bluetooth_messenger;
