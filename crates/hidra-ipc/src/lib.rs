#![deny(warnings)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::windows::named_pipe::NamedPipeClient,
};

pub const PIPE_PATH: &str = r"\\.\pipe\hidra";

#[derive(Debug, Serialize, Deserialize)]
pub struct PadState {
    pub buttons: u16,
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
    pub lt: u8,
    pub rt: u8,
    //TODO: add touchpad, motion, battery, etc
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum BrokerRequest {
    Ping,
    Create { kind: hidra_protocol::DeviceKind, features: u32 },
    Destroy { handle: u64 },
    UpdateState { handle: u64, state: PadState },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum BrokerResponse {
    Pong,
    OkCreate { handle: u64 },
    Ok,
    Err { message: String },
}

// === Client helpers ===

pub async fn connect_client() -> Result<NamedPipeClient> {
    ClientOptions::new()
        .open(PIPE_PATH)
        .with_context(|| format!("failed to open pipe {}", PIPE_PATH))
}

pub async fn read_json<T, R>(reader: &mut R) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
    R: tokio::io::AsyncRead + Unpin,
{
    let mut br = BufReader::new(reader);
    let mut line = String::new();
    let n = br.read_line(&mut line).await.context("read_line failed")?;
    if n == 0 {
        anyhow::bail!("peer closed pipe (eof)");
    }
    let value = serde_json::from_str(&line).context("invalid JSON frame")?;
    Ok(value)
}

pub async fn write_json<T, W>(writer: &mut W, value: &T) -> Result<()>
where
    T: serde::Serialize,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut bw = BufWriter::new(writer);
    let s = serde_json::to_string(value).context("serialize JSON failed")?;
    bw.write_all(s.as_bytes()).await.context("write failed")?;
    bw.write_all(b"\n").await.context("newline write failed")?;
    bw.flush().await.context("flush failed")?;
    Ok(())
}
