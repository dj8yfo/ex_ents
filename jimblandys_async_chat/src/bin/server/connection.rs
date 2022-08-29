use std::net;

use async_chat::utils::{self, ChatResult};
/// Handle a single client's connection.
use async_chat::{FromClient, FromServer};
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::Arc;

use crate::group_table::GroupTable;

pub async fn work_connection(
    socket: TcpStream,
    groups: Arc<GroupTable>,
) -> ChatResult<()> {
    let outbound = Arc::new(Outbound::new(socket.clone()));
    println!(
        "server/work_connection routine: initialized connection from client {}",
        outbound.id
    );

    let buffered = BufReader::new(socket);
    let from_client = utils::receive_as_json(buffered);

    let result = serve(from_client, outbound.clone(), groups).await;

    let shut_res = outbound.stream.lock().await.shutdown(net::Shutdown::Write);
    println!(
        "server/work_connection routine: dropping connection from  {}, shut_res {:?}",
        outbound.id, shut_res,
    );
    return result;
}

async fn serve(
    mut from_client: impl Stream<Item = ChatResult<FromClient>> + Unpin,
    outbound: Arc<Outbound>,
    groups: Arc<GroupTable>,
) -> ChatResult<()> {
    while let Some(request_result) = from_client.next().await {
        let request = request_result?;

        let result = match request {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join_and_leave_cycle(outbound.clone());
                Ok(())
            }

            FromClient::Post {
                group_name,
                message,
            } => match groups.get(&group_name) {
                Some(group) => {
                    group.post(message);
                    Ok(())
                }
                None => Err(format!("Group '{}' does not exist", group_name)),
            },
        };

        if let Err(message) = result {
            let report = FromServer::Error(message);
            outbound.send(report).await?;
        }
    }
    Ok(())
}
use async_std::sync::Mutex;

pub struct Outbound {
    pub (crate) id: std::net::SocketAddr,
    stream: Mutex<TcpStream>,
}

impl Outbound {
    pub fn new(to_client: TcpStream) -> Outbound {
        let id = to_client.peer_addr().unwrap();
        Outbound {
            stream: Mutex::new(to_client),
            id: id,
        }
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.stream.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}
