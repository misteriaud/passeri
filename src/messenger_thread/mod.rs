use std::{sync::mpsc::Receiver, error::Error};

use crate::{midi_thread::MidiPayload, rtp_midi};

pub enum Request<Addr> {
	// Receiver
	WaitForInvitation, // block on listening for invitation
	AcceptInvitation((Addr, rtp_midi::ControlPacket)), // initiator_token / accept invitation and forward all midi received

	// Sender
	InviteSomeone((Addr, rtp_midi::ControlPacket)), // send invitation to specified address:port
}

#[derive(Debug)]
pub enum Response<Addr> {
	// Receiver
	InvitationReceived((Addr, rtp_midi::ControlPacket)),
	StartReceiving,

	// Sender
	InvitationSended,
	StartSending,

	// Error
	Err
}

type Responder<Addr> = oneshot::Sender<Response<Addr>>;

pub trait Messenger {
	type Addr;
	fn new_sender(rx: Receiver<MidiPayload>, addr: Self::Addr) -> Self;
	fn req(&self, req: Request<Self::Addr>) -> Result<Response<Self::Addr>, Box<dyn Error>>;
	fn info(&self) -> String;
}

pub mod ip_messenger;
// pub mod bluetooth_messenger;
