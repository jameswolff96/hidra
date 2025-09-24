#![deny(warnings)]
use anyhow::Result;
use clap::Parser;
use hidra_client::{destroy, ping, spawn};
use hidra_protocol::DeviceKind;

#[derive(Parser)]
#[command(name = "hidra", about = "HIDra CLI tools")]
enum Cmd {
    Spawn {
        #[arg(value_enum)]
        kind: PadKind,
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
    match Cmd::parse() {
        Cmd::Spawn { kind } => {
            let kind = match kind {
                PadKind::X360 => DeviceKind::X360,
                PadKind::Ds4 => DeviceKind::DS4,
                PadKind::Ds5 => DeviceKind::DS5,
            };
            let h = spawn(kind).await?;
            println!("Spawned device with handle {}", h.0);
        }
        Cmd::Destroy { handle } => {
            destroy(hidra_client::GamepadHandle(handle)).await?;
            println!("Destroyed device with handle {}", handle);
        }
        Cmd::Ping => {
            ping().await?;
            println!("broker pong");
        }
    }
    Ok(())
}
