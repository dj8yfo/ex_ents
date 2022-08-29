//! A chat group.

use async_std::task;
use crate::{connection::Outbound, participants::GroupMembers};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Group {
    name: Arc<String>,
    participants: Arc<GroupMembers>,
    sender: broadcast::Sender<Arc<String>>
}
impl Group {
    pub fn new(name: Arc<String>, participants: Arc<GroupMembers>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        Group {
            name,
            participants,
            sender,
        }
    }

    pub fn join_and_leave_cycle(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();
        

        task::spawn(handle_subscriber(
            self.name.clone(),
            self.participants.clone(),
            receiver,
            outbound,
        ));
    }

    pub fn post(&self, message: Arc<String>) {
        // This only returns an error when there are no subscribers. A
        // connection's outgoing side can exit, dropping its subscription,
        // slightly before its incoming side, which may end up trying to send a
        // message to an empty group.
        let _ignored = self.sender.send(message);
    }
}
use async_chat::FromServer;
use tokio::sync::broadcast::error::RecvError;
async fn handle_subscriber(
    group_name: Arc<String>,
    participants: Arc<GroupMembers>,
    receiver: broadcast::Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    let member_id = outbound.id;
    if let Err(some) = participants.join(group_name.as_str(), member_id) {
        {
            println!("err on join: {}", some);
            return;
        }
    }
    loop_subscriber(group_name.clone(), receiver, outbound).await;
    participants.leave(&*group_name, member_id);
}

async fn loop_subscriber(
    group_name: Arc<String>,
    mut receiver: broadcast::Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    loop {
        let packet = match receiver.recv().await {
            Ok(message) => {
                println!("Ok(message) variant");
                FromServer::Message {
                    group_name: group_name.clone(),
                    message: message.clone(),
                }
            }

            Err(RecvError::Lagged(n)) => {
                println!("RecvError::Lagged variant");
                FromServer::Error(format!(
                "Dropped {} messages from {}.",
                n, group_name
            ))},

            Err(RecvError::Closed) => {
                println!(
                    "breaking on RecvError::Closed {}, {}",
                    group_name, outbound.id
                );
                break;
            }
        };

        if outbound.send(packet).await.is_err() {
            println!("breaking on send {}", outbound.id);
            break;
        }
    }
}
