#![deny(warnings)]
use anyhow::Result;
use clap::Parser;
use hidra_client::{destroy, ping, spawn};
use hidra_protocol::DeviceKind;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser)]
#[command(name = "hidra", about = "HIDra CLI tools")]
enum Cmd {
    Spawn {
        #[arg(value_enum)]
        kind: PadKind,
    },
    Update {
        #[arg(long)]
        handle: u64,
        #[arg(long)]
        state: String,
    },
    Destroy {
        handle: u64,
    },
    Ping,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum PadKind {
    X360,
    Ds4,
    Ds5,
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

    match Cmd::parse() {
        Cmd::Spawn { kind } => {
            let kind = match kind {
                PadKind::X360 => DeviceKind::X360,
                PadKind::Ds4 => DeviceKind::DS4,
                PadKind::Ds5 => DeviceKind::DS5,
            };
            let h = spawn(kind).await?;
            info!(handle = h.0, "spawned handle");
            println!("{}", h.0);
        }
        Cmd::Update { handle, state } => {
            let parts: Vec<&str> = state.split(',').collect();
            if parts.len() != 6 {
                anyhow::bail!("state must have 6 comma-separated values");
            }
            let s = serde_json::from_str(&state)?;
            info!(handle, state=?s, "updated state");

            hidra_client::update_state(hidra_client::GamepadHandle(handle), s).await?;
        }
        Cmd::Destroy { handle } => {
            destroy(hidra_client::GamepadHandle(handle)).await?;
            info!(handle, "destroyed handle");
        }
        Cmd::Ping => {
            ping().await?;
            info!("pong");
        }
    }
    Ok(())
}
