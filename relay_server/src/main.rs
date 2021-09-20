use nanoserde::{DeBin, SerBin};

use std::collections::HashMap;

#[derive(Debug, DeBin, SerBin)]
pub enum Message {
    Idle,
    RelayRequestId,
    RelayIdAssigned(u64),
    RelayConnectTo(u64),
    RelayConnected,
}

fn main() {
    let mut addr_to_id = HashMap::new();
    let mut id_to_addr = HashMap::new();
    let mut connections = HashMap::new();
    let mut id = 0;
    let socket = std::net::UdpSocket::bind("0.0.0.0:35000").unwrap();

    socket.set_nonblocking(true).unwrap();

    let mut buf = [0; 200];
    loop {
        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
            if !addr_to_id.contains_key(&addr) {
                addr_to_id.insert(addr, id);
                id_to_addr.insert(id, addr);
                id += 1;
            }

            match buf[0] {
                // Relay meta messages
                0 | 1 | 2 | 3 | 4 => {
                    let message: Message = DeBin::deserialize_bin(&buf[..]).unwrap();
                    match message {
                        Message::RelayRequestId => {
                            let id = addr_to_id[&addr];
                            let _ = socket.send_to(
                                &SerBin::serialize_bin(&Message::RelayIdAssigned(id)),
                                addr,
                            );
                        }
                        Message::RelayConnectTo(id) => {
                            let self_id = addr_to_id[&addr];
                            let other_addr = id_to_addr.get(&id).cloned();
                            if let Some(opponent) = other_addr {
                                connections.insert(self_id, opponent);
                            }
                            let _ = socket
                                .send_to(&SerBin::serialize_bin(&Message::RelayConnected), addr);
                        }
                        _ => {}
                    }
                }
                // Content message
                5 => {
                    let self_id = addr_to_id[&addr];

                    // if there is a connection established for this client
                    if let Some(opponent) = connections.get(&self_id) {
                        let _ = socket.send_to(&buf, opponent);
                    }
                }
                // Unsupported, unknown message
                _ => panic!(),
            }
        }
    }
}
