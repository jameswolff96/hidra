#![deny(warnings)]

pub mod backend;

use anyhow::Result;
use hidra_ipc::{BrokerRequest, BrokerResponse, PIPE_PATH, read_json_opt, write_json};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tracing::{error, info, instrument};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[cfg(feature = "backend-driver")]
use crate::backend::Driver;
use crate::backend::{Backend, mock::Mock};

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
        let server = ServerOptions::new().create(PIPE_PATH)?;
        #[cfg(feature = "backend-driver")]
        let backend = Driver::new();
        #[cfg(not(feature = "backend-driver"))]
        let backend = Mock::new();

        server.connect().await?;
        info!("client connected");

        tokio::spawn(async move {
            if let Err(e) = serve_connected(server, backend, nh).await {
                error!(error=%e, "client session error");
            }
        });

        // 4) Loop back to create the NEXT instance
    }
}

#[instrument(skip(server, backend, next_handle), fields(peer=?std::thread::current().id()))]
async fn serve_connected(
    mut server: NamedPipeServer,
    backend: Arc<dyn Backend>,
    next_handle: Arc<AtomicU64>,
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
                    Ok(_) => {
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
                info!(handle, ?state, "update state");
                match backend.update(handle, state.try_into()?).await {
                    Ok(_) => {
                        info!(handle, "updated state");
                        write_json(&mut server, &BrokerResponse::Ok).await?;
                    }
                    Err(e) => {
                        error!(error=%e, "backend update error");
                        write_json(&mut server, &BrokerResponse::Err { message: e.to_string() })
                            .await?;
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
