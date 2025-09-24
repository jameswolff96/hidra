#![deny(warnings)]

use anyhow::Result;
use hidra_ipc::{BrokerRequest, BrokerResponse, PIPE_PATH, read_json, write_json};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

#[tokio::main]
async fn main() -> Result<()> {
    println!("[hidra-broker] starting on {PIPE_PATH}");

    let next_handle = Arc::new(AtomicU64::new(1));

    loop {
        let server = ServerOptions::new().create(PIPE_PATH)?;
        let nh = next_handle.clone();

        tokio::spawn(async move {
            if let Err(e) = serve_client(server, nh).await {
                eprintln!("[hidra-broker] client error: {e:#}");
            }
        });

        // Loop to create the next pipe instance; the spawned task owns this one.
    }
}

async fn serve_client(mut server: NamedPipeServer, next_handle: Arc<AtomicU64>) -> Result<()> {
    server.connect().await?;
    println!("[hidra-broker] client connected");

    loop {
        match read_json::<BrokerRequest, _>(&mut server).await {
            Ok(BrokerRequest::Create { kind: _kind, features: _features }) => {
                let handle = next_handle.fetch_add(1, Ordering::SeqCst);
                write_json(&mut server, &BrokerResponse::OkCreate { handle }).await?;
            }
            Ok(BrokerRequest::Destroy { handle: _ }) => {
                write_json(&mut server, &BrokerResponse::Ok).await?;
            }
            Ok(BrokerRequest::Ping) => {
                write_json(&mut server, &BrokerResponse::Pong).await?;
            }
            Ok(BrokerRequest::UpdateState { handle: _, state: _ }) => {
                // TODO: Forward to driver
                write_json(&mut server, &BrokerResponse::Ok).await?;
            }
            Err(e) => {
                // Send error back (best effort) then exit.
                let _ =
                    write_json(&mut server, &BrokerResponse::Err { message: e.to_string() }).await;
                let _ = server.flush().await;
                break;
            }
        }
    }

    println!("[hidra-broker] client disconnected");

    Ok(())
}
