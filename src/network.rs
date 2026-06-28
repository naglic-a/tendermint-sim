use crate::types::Message;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn send_message(stream: &mut TcpStream, msg: &Message) -> io::Result<()> {
    let serialized = serde_json::to_vec(msg)?;
    let len = serialized.len() as u32;
    stream.write_u32(len).await?;
    stream.write_all(&serialized).await?;
    Ok(())
}

pub async fn receive_message(stream: &mut TcpStream) -> io::Result<Option<Message>> {
    let len = match stream.read_u32().await {
        Ok(len) => len,
        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };

    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;

    let msg: Message = serde_json::from_slice(&buf)?;
    Ok(Some(msg))
}
