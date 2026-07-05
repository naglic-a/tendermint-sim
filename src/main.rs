pub mod consensus;
pub mod types;
pub mod perfect_link;

use crate::types::{Event, PlRequest, Message, Proposal, Value};
use crate::perfect_link::PerfectLink;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::collections::{HashMap, HashSet};
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    info!("Test Test Test");

    let(req_sender, req_receiver) = mpsc::channel(100);
    let(event_sender, mut event_receiver) = mpsc::channel(100);
    let(internal_sender, internal_receiver) = mpsc::channel(100);

    let mut perfect_link = PerfectLink::new(
        req_receiver,
        event_sender.clone(),
        internal_receiver,
        HashMap::new(),
        HashSet::new(),
    );

    perfect_link.start_listener(8000, internal_sender).await;

    tokio::spawn(async move {
        perfect_link.run().await;
    });

    let mut fake_peer = TcpStream::connect("127.0.0.1:8000").await.unwrap();

    let test_msg = Message::Proposal(Proposal {
        height: 1,
        round: 1,
        value: Value { data: "TestValue".to_string() },
        valid_round: None,
        sender: 99,
    });
    let bytes = serde_json::to_vec(&test_msg).unwrap();

    fake_peer.write_u32(bytes.len() as u32).await.unwrap();
    fake_peer.write_all(&bytes).await.unwrap();

    if let Some(event) = event_receiver.recv().await {
        println!("its working!!! Event received: {:?}", event);
    }
}
