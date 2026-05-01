use std::sync::mpsc::{Receiver, Sender};
use crate::data::{TicketDraft};
use crate::store::{TicketStore};

pub mod data;
pub mod store;

#[derive(Debug)]
pub enum Command {
    Insert(TicketDraft),
}

// Start the system by spawning the server thread.
// It returns a `Sender` instance which can then be used
// by one or more clients to interact with the server.
pub fn launch() -> Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || server(receiver));
    sender
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();

    for command in receiver {
        match command {
            Command::Insert(ticket_draft) => {
                store.add_ticket(ticket_draft);
            }
        }
    }
}
