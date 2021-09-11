//! Very, very WIP
//! "Delayed lockstep" networking implementation - first step towards GGPO

use macroquad::experimental::scene::{self, Handle, Node, NodeWith, RefMut};

use crate::{
    capabilities::NetworkReplicate,
    input::{self, Input, InputScheme},
    nodes::Player,
};

use std::{net::UdpSocket, sync::mpsc};

use nanoserde::{DeBin, SerBin};

#[derive(DeBin, SerBin)]
enum Message {
    Input {
        // position in the buffer
        pos: u8,
        // current simulation frame
        frame: u64,
        input: Input,
    },
    Ack {
        // last simulated frame
        frame: u64,
        /// bitmask for last 8 frames
        /// one bit - one received input in a frames_buffer
        ack: u8,
    },
}

pub struct Network {
    input_scheme: InputScheme,

    player1: Handle<Player>,
    player2: Handle<Player>,

    self_id: usize,

    ack_frame: u64,

    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,

    frames_buffer: [[Option<Input>; 2]; Self::CONSTANT_DELAY as usize],
}

// get a bitmask of received remote inputs out of frames_buffer
fn remote_inputs_ack(
    remote_player_id: usize,
    buffer: &[[Option<Input>; 2]; Network::CONSTANT_DELAY as usize],
) -> u8 {
    let mut ack = 0;

    #[allow(clippy::needless_range_loop)]
    for i in 0..Network::CONSTANT_DELAY as usize {
        if buffer[i][remote_player_id].is_some() {
            ack |= 1 << i;
        }
    }
    ack
}

impl Network {
    /// 8-bit bitmask is used for ACK, to make CONSTANT_DELAY more than 8
    /// bitmask type should be changed
    const CONSTANT_DELAY: usize = 8;

    pub fn new(
        input_scheme: InputScheme,
        player1: Handle<Player>,
        player2: Handle<Player>,
        controller_id: usize,
        self_addr: &str,
        other_addr: &str,
    ) -> Network {
        let self_socket = UdpSocket::bind(self_addr).unwrap();
        self_socket.connect(other_addr).unwrap();

        let (tx, rx) = mpsc::channel::<Message>();

        let (tx1, rx1) = mpsc::channel::<Message>();

        {
            let self_socket = self_socket.try_clone().unwrap();
            std::thread::spawn(move || {
                let self_socket = self_socket;
                loop {
                    let mut data = [0; 256];
                    match self_socket.recv_from(&mut data) {
                        Err(..) => {} //println!("waiting for other player"),
                        Ok((count, _)) => {
                            assert!(count < 256);
                            let message = DeBin::deserialize_bin(&data[0..count]).unwrap();
                            std::thread::sleep(std::time::Duration::from_millis(
                                macroquad::rand::gen_range(0, 10),
                            ));

                            tx1.send(message).unwrap();
                        }
                    }
                }
            });
        }

        let other_addr = other_addr.to_owned();
        std::thread::spawn(move || {
            let other_addr = other_addr.to_owned();
            loop {
                if let Ok(message) = rx.recv() {
                    let data = SerBin::serialize_bin(&message);
                    let _ = self_socket.send_to(&data, &other_addr);
                }
            }
        });

        let mut frames_buffer = [[None, None]; Self::CONSTANT_DELAY];

        // Fill first CONSTANT_DELAY frames
        // this will not really change anything - the fish will just always spend
        // first CONSTANT_DELAY frames doing nothing, not a big deal
        // But with pre-filled buffer we can avoid any special-case logic
        // at the start of the game and later on will just wait for remote
        // fish to fill up their part of the buffer
        #[allow(clippy::needless_range_loop)]
        for i in 0..Self::CONSTANT_DELAY {
            frames_buffer[i][controller_id as usize] = Some(Input::default());
        }

        Network {
            input_scheme,
            self_id: controller_id,
            player1,
            player2,
            ack_frame: 0,
            tx,
            rx: rx1,
            frames_buffer,
        }
    }
}

impl Node for Network {
    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        let own_input = input::collect_input(node.input_scheme);

        // Right now there are only two players, so it is possible to find out
        // remote fish id as "not ours" id. With more fish it will be more complicated
        // and ID will be part of a protocol
        let remote_id = if node.self_id == 1 { 0 } else { 1 };

        // go through the whole frames_buffer and re-send frames missing on remote fish
        for i in 0..Self::CONSTANT_DELAY {
            if node.frames_buffer[i][remote_id].is_none() {
                node.tx
                    .send(Message::Input {
                        frame: node.ack_frame,
                        pos: i as u8,
                        input: node.frames_buffer[i][node.self_id].unwrap(),
                    })
                    .unwrap();
            }
        }

        // Receive other fish input
        let mut remote_ack = 0;
        while let Ok(message) = node.rx.try_recv() {
            match message {
                Message::Input { pos, frame, input } => {
                    // frame may come from the past and from the future, so
                    // we do frame-corrective here
                    let ix = pos as i64 + (frame as i64 - node.ack_frame as i64);

                    // if packet is outside of CONSTANT_DELAY window - just skip it
                    // it will be resend anyway
                    if ix >= 0 && ix < Self::CONSTANT_DELAY as i64 {
                        node.frames_buffer[ix as usize][remote_id] = Some(input);
                    }
                }
                Message::Ack { frame, ack } => {
                    // ack is from another frame, the buffer was shifted
                    // a few times on the remote fish
                    // shift their ack back to our timeline
                    let shift = frame as i64 - node.ack_frame as i64;

                    // otherwise ack is from non-relevant frame
                    // maybe some random packet just got received
                    if shift.abs() < Self::CONSTANT_DELAY as _ {
                        println!("{} {}", ack, shift);
                        let ack = if shift > 0 {
                            ack << shift
                        } else {
                            ack >> (-shift)
                        };
                        remote_ack |= ack as u8;
                    }
                }
            }
        }

        // notify the other fish on the state of our input buffer
        node.tx
            .send(Message::Ack {
                frame: node.ack_frame,
                ack: remote_inputs_ack(remote_id, &node.frames_buffer),
            })
            .unwrap();

        if remote_ack & 1 == 0 {
            //println!("Not enough inputs received, pausing game to wait");
        }

        // we have an input for "-CONSTANT_DELAY" frame, so we can
        // advance the simulation
        if remote_ack & 1 == 1 {
            let [p1_input, p2_input] = node.frames_buffer[0];
            assert!(p1_input.is_some());
            assert!(p2_input.is_some());

            let p1_input = p1_input.unwrap();
            let p2_input = p2_input.unwrap();

            scene::get_node(node.player1).apply_input(p1_input);
            scene::get_node(node.player2).apply_input(p2_input);

            // advance the simulation
            for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
                (capability.network_update)(node);
            }

            // this input frame is processed, so shifting input buffer
            for i in 1..Self::CONSTANT_DELAY as usize {
                node.frames_buffer[i - 1] = node.frames_buffer[i];
            }

            // and inserting own input to buffer's last frame
            node.frames_buffer[Self::CONSTANT_DELAY - 1][node.self_id] = Some(own_input);
            // clean up the other fish input
            node.frames_buffer[Self::CONSTANT_DELAY - 1][remote_id] = None;

            node.ack_frame += 1;
        }
    }
}
