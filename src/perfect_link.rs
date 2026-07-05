use crate::types::{PlRequest, Event, Message, NodeId};
use tokio::sync::mpsc;
use std::collections::HashMap;
use tokio::net::TcpListener;
use std::collections::HashSet;
use sha2::{Sha256, Digest};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub struct PerfectLink {
    // Consensus -> Network
    request_receiver: mpsc::Receiver<PlRequest>,
    // Network -> Consensus
    event_sender: mpsc::Sender<Event>, 
    internal_receiver: mpsc::Receiver<Vec<u8>>,

    peers: HashMap<NodeId, OwnedWriteHalf>,
    seen_messages: HashSet<String>,
}

impl PerfectLink {
    pub fn new(
        request_receiver: mpsc::Receiver<PlRequest>,
        event_sender: mpsc::Sender<Event>,
        internal_receiver: mpsc::Receiver<Vec<u8>>,
        peers: HashMap<NodeId, OwnedWriteHalf>,
        seen_messages: HashSet<String>,
    ) -> Self {
        PerfectLink {
            request_receiver,
            event_sender,
            internal_receiver,
            peers,
            seen_messages,
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(request) = self.request_receiver.recv() => {
                    match request {
                        PlRequest::Send { dest, msg } => {
                            if let Some(peer_socket) = self.peers.get_mut(&dest) {
                                let raw_bytes = serde_json::to_vec(&msg).unwrap();
                                let len = raw_bytes.len() as u32;

                                let _ = peer_socket.write_u32(len).await;
                                let _ = peer_socket.write_all(&raw_bytes).await;
                            }
                        }
                        PlRequest::Broadcast { msg } => {
                            let raw_bytes = serde_json::to_vec(&msg).unwrap();
                            let len = raw_bytes.len() as u32;

                            for(node_id, peer_socket) in self.peers.iter_mut() {
                                let _ = peer_socket.write_u32(len).await;
                                let _ = peer_socket.write_all(&raw_bytes).await;
                            }
                        }
                    }
                }

                Some(raw_bytes) = self.internal_receiver.recv() => {
                    // TODO : 1. Compute SHA-256 hash of the raw_bytes
                    // TODO : 2. If its allready in self.seen_messages, do nothing
                    // TODO: 3. if its new, add hash to self.seen_messages
                    // TODO: 4. Deserialize the raw_bytes into a Message
                    // TODO: 5. Forwared the message to the consensus layer by sending an Event::PlDeliver to self.event_sender
                    // TODO: 6. Broadcast the message to all other nodes by sending an Event::PlDeliver to self.event_sender
                    let mut hasher = Sha256::new();
                    hasher.update(&raw_bytes);
                    let hash_result = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

                    if self.seen_messages.contains(&hash_result) {
                        continue;
                    }
                    self.seen_messages.insert(hash_result);

                    let msg: Message = match serde_json::from_slice(&raw_bytes) {
                        Ok(m) => m,
                        Err(_) => {
                            continue; 
                        }
                    };

                    let src = match &msg {
                        Message::Proposal(proposal) => proposal.sender,
                        Message::Vote(vote) => vote.sender,
                    };

                    self.event_sender.send(Event::PlDeliver { src, msg }).await.unwrap();
                    
                    let len = raw_bytes.len() as u32;
                    for(node_id, peer_socket) in self.peers.iter_mut() {
                        let _ = peer_socket.write_u32(len).await;
                        let _ = peer_socket.write_all(&raw_bytes).await;
                    }
                }
            }
        }
    }   

    pub async fn start_listener(&self, port: u16, internal_sender: mpsc::Sender<Vec<u8>>) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();  

        tokio::spawn(async move {
            loop {
                let(mut socket, _addr) = listener.accept().await.unwrap();
                let sender_clone = internal_sender.clone();

                tokio::spawn(async move {
                    loop {
                        // TODO : 1. Read Bytes from the socket
                        // TODO : 2. Send the bytes to the brain
                        // internall_seender.send(buf).await.unwrap();
                        let len = match socket.read_u32().await {
                            Ok(l) => l as usize,
                            Err(_) => break, // Connection closed or error occurred
                        };

                        let mut buf = vec![0u8; len];
                        if let Err(_) = socket.read_exact(&mut buf).await {
                            break; // Connection closed or error occurred
                        }

                        if sender_clone.send(buf).await.is_err() {
                            break; // drop loop if main crashed
                        }
                    }
                });
            }
        });
    }
}


