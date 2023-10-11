use std::{sync::mpsc::Receiver, error::Error};

use crate::midi_thread::MidiPayload;

pub enum Request {
	// Receiver
	WaitForInvitation, // block on listening for invitation
	AcceptInvitation(u32), // initiator_token / accept invitation and forward all midi received
	// Sender
	InviteSomeone(String), // send invitation to specified address:port
}

#[derive(Debug)]
pub enum Response {
	// Receiver
	InvitationReceived{
		initiator_token: u32,
		ssrc: u32,
		name: String,
	},
	StartReceiving,

	// Sender
	InvitationSended,
	StartSending
}

type Responder = oneshot::Sender<Response>;

pub trait Messenger {
	fn new_sender(rx: Receiver<MidiPayload>) -> Self;
	fn req(&self, req: Request) -> Result<Response, Box<dyn Error>>;
	fn info(&self) -> String;
}

pub mod ip_messenger;
