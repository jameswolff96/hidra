#![deny(warnings)]
use anyhow::Result;
use clap::Parser;
use hidra_client::{GamepadHandle, destroy, ping, spawn, update_state};
use hidra_ipc::PadState;
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
        state_json: Option<String>,
        #[arg(long)]
        buttons: Option<u16>,
        #[arg(long)]
        lx: Option<i16>,
        #[arg(long)]
        ly: Option<i16>,
        #[arg(long)]
        rx: Option<i16>,
        #[arg(long)]
        ry: Option<i16>,
        #[arg(long)]
        lt: Option<u16>,
        #[arg(long)]
        rt: Option<u16>,
    },
    Destroy {
        handle: u64,
    },
    Ping,
    QuickProbe,
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
        Cmd::Update { handle, state_json, buttons, lx, ly, rx, ry, lt, rt } => {
            let mut s = if let Some(j) = state_json {
                serde_json::from_str(&j)?
            } else {
                PadState::default()
            };

            if let Some(v) = buttons {
                s.buttons = v
            }
            if let Some(v) = lx {
                s.lx = v
            }
            if let Some(v) = ly {
                s.ly = v
            }
            if let Some(v) = rx {
                s.rx = v
            }
            if let Some(v) = ry {
                s.ry = v
            }
            if let Some(v) = lt {
                s.lt = v
            }
            if let Some(v) = rt {
                s.rt = v
            }

            info!(handle, state=?s, "updated state");

            update_state(GamepadHandle(handle), s).await?;
        }
        Cmd::Destroy { handle } => {
            destroy(GamepadHandle(handle)).await?;
            info!(handle, "destroyed handle");
        }
        Cmd::Ping => {
            ping().await?;
            info!("pong");
        }
        Cmd::QuickProbe => {
            let h = spawn(DeviceKind::DS4).await?;
            info!(handle = h.0, "spawned handle");

            let state = PadState { rx: 5, ..Default::default() };
            update_state(h, state.clone()).await?;
            info!(handle = h.0, ?state, "updated state");

            destroy(h).await?;
            info!(handle = h.0, "destroyed handle")
        }
    }
    Ok(())
}
