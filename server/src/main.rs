use nanoserde::{DeBin, SerBin};
use std::net::UdpSocket;

use server::MetaMessage;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:34500").expect("couldn't bind to address");

    let mut opponent: Option<String> = None;

    loop {
        let mut buf = [0; 100];
        let (_, addr) = socket.recv_from(&mut buf).expect("Didn't receive data");

        let message: MetaMessage = DeBin::deserialize_bin(&buf[..]).unwrap();
        assert_eq!(message, MetaMessage::ConnectionRequest);

        println!("Player connected: {}", addr);

        if let Some(op) = opponent {
            println!("We have two players, matching {} with {}", addr, op);

            // send opponent IP to current connection
            let buf = SerBin::serialize_bin(&MetaMessage::OpponentIp {
                id: 0,
                ip: addr.to_string(),
            });
            socket.send_to(&buf, &op).unwrap();

            // and send current connection IP to the opponent
            let buf = SerBin::serialize_bin(&MetaMessage::OpponentIp {
                id: 1,
                ip: op.clone(),
            });
            socket.send_to(&buf, &addr).unwrap();

            opponent = None;
        } else {
            opponent = Some(format!("{}", addr));
        }
    }
}
