// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, String> {
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(5);
        let insert_command = Command::Insert {
            draft: draft,
            response_channel: response_sender,
        };
        match self.sender.try_send(insert_command) {
            Ok(_) => {}
            Err(_) => {
                return Err("Channel is full, insert command cannot be sent from client".to_string())
            }
        }
        let ticket_id: TicketId = response_receiver.recv().expect("No response received!");
        Ok(ticket_id)
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, String> {
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(5);
        let get_command = Command::Get {
            id: id,
            response_channel: response_sender,
        };
        match self.sender.try_send(get_command) {
            Ok(_) => {}
            Err(e) => {
                return Err("Channel is full, get command cannot be sent from client".to_string())
            }
        }
        let ticket: Option<Ticket> = response_receiver.recv().expect("No response received!");
        Ok(ticket)
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>,
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
                response_channel
                    .try_send(id)
                    .expect("Channel is full, ticket id cannot be sent from server");
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                response_channel
                    .try_send(ticket.cloned())
                    .expect("Channel is full, ticket cannot be sent from server");
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
