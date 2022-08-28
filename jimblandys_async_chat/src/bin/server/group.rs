//! A chat group.

use async_std::{task, sync::Mutex};
use crate::connection::Outbound;
use std::{sync::Arc, collections::HashSet, net::SocketAddr};
use tokio::sync::broadcast;

pub struct Group {
    name: Arc<String>,
    participants: Arc<Mutex<HashSet<SocketAddr>>>,
    sender: broadcast::Sender<Arc<String>>
}
impl Group {
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        let participants = Arc::new(Mutex::new(HashSet::new()));
        Group {
            name,
            participants,
            sender,
        }
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
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
    participants: Arc<Mutex<HashSet<SocketAddr>>>,
    receiver: broadcast::Receiver<Arc<String>>,
    outbound: Arc<Outbound>,
) {
    let participant_id = outbound.id;
    {
        let mut guard = participants.lock().await;
        if !guard.insert(outbound.id) {
            println!(
                "double join attempt {} <- {}",
                group_name, participant_id
            );
            return;
        } else {
            println!("joined {} <- {}", group_name, outbound.id);
        }
    }
    loop_subscriber(group_name.clone(), receiver, outbound).await;
    {
        let mut guard = participants.lock().await;
        let removed = guard.remove(&participant_id);
        println!(
            "removed from {} ->out  \"{}\", if removed {}",
            group_name, participant_id, removed
        );
    }
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
