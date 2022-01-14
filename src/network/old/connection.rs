use std::net::UdpSocket;

const RELAY_ADDR: &str = "173.0.157.169:35000";

use super::NetworkMessage;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkConnectionKind {
    Lan,
    Stun,
    Relay,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum NetworkConnectionStatus {
    Unknown,
    Connected,
}

pub struct NetworkConnection {
    pub kind: NetworkConnectionKind,
    pub socket: Option<UdpSocket>,
    pub local_addr: String,
    pub opponent_addr: String,
    pub relay_addr: String,
    pub status: NetworkConnectionStatus,
}

impl Default for NetworkConnection {
    fn default() -> Self {
        NetworkConnection::new()
    }
}

impl NetworkConnection {
    pub fn new() -> NetworkConnection {
        NetworkConnection {
            kind: NetworkConnectionKind::Unknown,
            socket: None,
            local_addr: "".to_string(),
            opponent_addr: "".to_string(),
            relay_addr: RELAY_ADDR.to_string(),
            status: NetworkConnectionStatus::Unknown,
        }
    }

    pub fn update(&mut self, kind: NetworkConnectionKind) {
        if let Some(socket) = self.socket.as_mut() {
            let mut buf = [0; 100];
            if socket.recv(&mut buf).is_ok() {
                let _message: NetworkMessage =
                    nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                self.status = NetworkConnectionStatus::Connected;
            }
        }

        if kind != self.kind {
            self.kind = kind;
            self.status = NetworkConnectionStatus::Unknown;

            use std::net::SocketAddr;

            let addrs = [
                SocketAddr::from(([0, 0, 0, 0], 3400)),
                SocketAddr::from(([0, 0, 0, 0], 3401)),
                SocketAddr::from(([0, 0, 0, 0], 3402)),
                SocketAddr::from(([0, 0, 0, 0], 3403)),
            ];

            match kind {
                NetworkConnectionKind::Lan => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();

                    self.local_addr = format!("{}", socket.local_addr().unwrap());
                    socket.set_nonblocking(true).unwrap();

                    self.socket = Some(socket);
                }
                NetworkConnectionKind::Stun => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();

                    let sc = stunclient::StunClient::with_google_stun_server();
                    self.local_addr = format!("{}", sc.query_external_address(&socket).unwrap());
                    socket.set_nonblocking(true).unwrap();

                    self.socket = Some(socket);
                }
                NetworkConnectionKind::Relay => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();
                    socket.connect(&self.relay_addr).unwrap();
                    socket.set_nonblocking(true).unwrap();

                    loop {
                        let _ = socket.send(&nanoserde::SerBin::serialize_bin(
                            &NetworkMessage::RelayRequestId,
                        ));

                        let mut buf = [0; 100];
                        if socket.recv(&mut buf).is_ok() {
                            let message: NetworkMessage =
                                nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                            if let NetworkMessage::RelayIdAssigned(id) = message {
                                self.local_addr = format!("{}", id);
                                break;
                            }
                        }
                    }
                    self.socket = Some(socket);
                }
                _ => {}
            }
        }
    }

    pub fn connect(&mut self) -> bool {
        let socket = self.socket.as_mut().unwrap();
        match self.kind {
            NetworkConnectionKind::Lan | NetworkConnectionKind::Stun => {
                socket.connect(&self.opponent_addr).unwrap();
            }
            NetworkConnectionKind::Relay => {
                let other_id;
                match self.opponent_addr.parse::<u64>() {
                    Ok(v) => other_id = v,
                    Err(_) => return false,
                };
                loop {
                    let _ = socket.send(&nanoserde::SerBin::serialize_bin(
                        &NetworkMessage::RelayConnectTo(other_id),
                    ));

                    let mut buf = [0; 100];
                    if socket.recv(&mut buf).is_ok() {
                        let message: NetworkMessage =
                            nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                        if let NetworkMessage::RelayConnected = message {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        true
    }

    pub fn probe(&mut self) -> Option<()> {
        assert!(self.socket.is_some());

        if self.kind == NetworkConnectionKind::Relay {
            return Some(());
        }
        let socket = self.socket.as_mut().unwrap();

        socket.connect(&self.opponent_addr).ok()?;

        for _ in 0..100 {
            socket
                .send(&nanoserde::SerBin::serialize_bin(&NetworkMessage::Idle))
                .ok()?;
        }

        None
    }
}
