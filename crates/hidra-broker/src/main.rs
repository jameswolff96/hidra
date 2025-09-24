#![deny(warnings)]

pub mod backend;

use anyhow::Result;
use hidra_ipc::{BrokerRequest, BrokerResponse, PIPE_PATH, read_json_opt, write_json};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tracing::{error, info, instrument};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_ansi(true)
        .compact()
        .init();

    info!(pipe=%PIPE_PATH, "hidra-broker starting");

    let next_handle = Arc::new(AtomicU64::new(1));

    loop {
        // 1) Create ONE instance
        let server = ServerOptions::new().create(PIPE_PATH)?;
        // 2) Wait for a client to connect BEFORE spawning
        server.connect().await?;
        info!("client connected");

        let nh = next_handle.clone();

        // 3) Move the *connected* server into a task to serve it
        tokio::spawn(async move {
            if let Err(e) = serve_connected(server, nh).await {
                error!(error=%e, "client session error");
            }
        });

        // 4) Loop back to create the NEXT instance
    }
}

#[instrument(skip(server, next_handle), fields(peer=?std::thread::current().id()))]
async fn serve_connected(mut server: NamedPipeServer, next_handle: Arc<AtomicU64>) -> Result<()> {
    loop {
        match read_json_opt::<BrokerRequest, _>(&mut server).await {
            Ok(None) => {
                info!("client disconnected");
                break;
            }
            Ok(Some(BrokerRequest::Create { kind, features })) => {
                let handle = next_handle.fetch_add(1, Ordering::SeqCst);
                info!(?kind, features, handle, "create device");
                write_json(&mut server, &BrokerResponse::OkCreate { handle }).await?;
            }
            Ok(Some(BrokerRequest::Destroy { handle })) => {
                info!(handle, "destroy device");
                write_json(&mut server, &BrokerResponse::Ok).await?;
            }
            Ok(Some(BrokerRequest::Ping)) => {
                write_json(&mut server, &BrokerResponse::Pong).await?;
            }
            Ok(Some(BrokerRequest::UpdateState { handle, state })) => {
                info!(handle, ?state, "update state");
                // TODO: Forward to driver
                write_json(&mut server, &BrokerResponse::Ok).await?;
            }
            Err(e) => {
                // Send error back (best effort) then exit.
                error!(error=%e, "protocol/read error");
                let _ =
                    write_json(&mut server, &BrokerResponse::Err { message: e.to_string() }).await;
                let _ = server.flush().await;
                break;
            }
        }
    }
    Ok(())
}
