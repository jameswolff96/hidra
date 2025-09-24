#![deny(warnings)]

pub mod backend;

use anyhow::Result;
use dashmap::DashMap;
use hidra_ipc::{BrokerRequest, BrokerResponse, PIPE_PATH, read_json_opt, write_json};
use hidra_protocol::ioctl;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::watch;
use tokio::time;
use tracing::{error, info, instrument};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[cfg(feature = "backend-driver")]
use crate::backend::Driver;
use crate::backend::{Backend, mock::Mock};

#[derive(Default)]
struct Pumps {
    map: DashMap<u64, watch::Sender<ioctl::PadState>>,
}

#[instrument(level = "debug", skip(backend, rx))]
async fn run_pump(
    backend: Arc<dyn Backend>,
    handle: u64,
    mut rx: watch::Receiver<ioctl::PadState>,
) {
    let mut dirty = true;
    let mut tick = time::interval(time::Duration::from_millis(4));
    loop {
        tokio::select! {
            _ = rx.changed() => { dirty = true; }
            _ = tick.tick() => {
                if dirty {
                    let cur = *rx.borrow();
                    if let Err(e) = backend.update(handle, cur).await {
                        error!(handle, error=%e, "backend.update failed");
                    }
                    dirty = false;
                }
            }
        }
    }
}

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

    #[cfg(feature = "backend-driver")]
    let backend: Arc<dyn Backend> = Driver::new();
    #[cfg(not(feature = "backend-driver"))]
    let backend: Arc<dyn Backend> = Mock::new();
    let pumps = Arc::new(Pumps::default());

    loop {
        let server = ServerOptions::new().create(PIPE_PATH)?;
        let backend = backend.clone();
        let pumps = pumps.clone();

        server.connect().await?;
        info!("client connected");

        tokio::spawn(async move {
            if let Err(e) = serve_connected(server, backend, pumps).await {
                error!(error=%e, "client session error");
            }
        });

        // 4) Loop back to create the NEXT instance
    }
}

#[instrument(skip(server, backend, pumps), fields(peer=?std::thread::current().id()))]
async fn serve_connected(
    mut server: NamedPipeServer,
    backend: Arc<dyn Backend>,
    pumps: Arc<Pumps>,
) -> Result<()> {
    loop {
        match read_json_opt::<BrokerRequest, _>(&mut server).await {
            Ok(None) => {
                info!("client disconnected");
                break;
            }
            Ok(Some(BrokerRequest::Create { kind, features })) => {
                info!(?kind, features, "create device");
                match backend.create(kind, features).await {
                    Ok(handle) => {
                        let (tx, rx) =
                            watch::channel::<ioctl::PadState>(ioctl::PadState::default());
                        pumps.map.insert(handle, tx);
                        let b = backend.clone();
                        tokio::spawn(run_pump(b, handle, rx));
                        info!(handle, "created device");
                        write_json(&mut server, &BrokerResponse::OkCreate { handle }).await?;
                    }
                    Err(e) => {
                        error!(error=%e, "backend create error");
                        write_json(&mut server, &BrokerResponse::Err { message: e.to_string() })
                            .await?;
                    }
                }
            }
            Ok(Some(BrokerRequest::Destroy { handle })) => {
                info!(handle, "destroy device");
                let _ = pumps.map.remove(&handle);
                match backend.destroy(handle).await {
                    Ok(_) => {
                        info!(handle, "destroyed device");
                        write_json(&mut server, &BrokerResponse::Ok).await?;
                    }
                    Err(e) => {
                        error!(error=%e, "backend destroy error");
                        write_json(&mut server, &BrokerResponse::Err { message: e.to_string() })
                            .await?;
                    }
                }
            }
            Ok(Some(BrokerRequest::Ping)) => {
                write_json(&mut server, &BrokerResponse::Pong).await?;
            }
            Ok(Some(BrokerRequest::UpdateState { handle, state })) => {
                let s: ioctl::PadState = state.try_into()?;
                if let Some(tx) = pumps.map.get(&handle) {
                    let _ = tx.value().send(s);
                    write_json(&mut server, &BrokerResponse::Ok).await?;
                } else {
                    match backend.update(handle, s).await {
                        Ok(_) => {
                            info!(handle, "updated state");
                            write_json(&mut server, &BrokerResponse::Ok).await?;
                        }
                        Err(e) => {
                            error!(error=%e, "backend update error");
                            write_json(
                                &mut server,
                                &BrokerResponse::Err { message: e.to_string() },
                            )
                            .await?;
                        }
                    }
                }
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
