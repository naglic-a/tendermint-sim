pub mod types;
pub mod network;
pub mod consensus;

use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Test Test Test");
}
