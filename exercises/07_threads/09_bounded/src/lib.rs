// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{Receiver, Sender, SyncSender, RecvError};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>
}

impl TicketStoreClient {
    fn new(sender: SyncSender<Command>) -> Self {
        Self {
            sender
        }
    }

    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, RecvError> {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);

        self.sender.try_send(Command::Insert {
            draft,
            response_channel: sender
        }).expect("Unable to insert");

        receiver.recv()
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, RecvError> {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);

        self.sender.try_send(Command::Get {
            id,
            response_channel: sender
        }).expect("Unable to get");

        receiver.recv()
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(10);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient::new(sender)
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                response_channel.try_send(id).expect("Unable to insert");
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                response_channel.try_send(ticket.cloned()).expect("Unable to get");
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
