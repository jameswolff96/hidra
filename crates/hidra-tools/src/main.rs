#![deny(warnings)]
use anyhow::Result;
use clap::Parser;
use hidra_client::{set_rumble, spawn};
use hidra_protocol::DeviceKind;

#[derive(Parser)]
#[command(name="hidra", about="HIDra CLI tools")]
enum Cmd {
Spawn { #[arg(value_enum)] kind: PadKind },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum PadKind { X360, Ds4, Ds5 }

#[tokio::main]
async fn main() -> Result<()> {
match Cmd::parse() {
Cmd::Spawn { kind } => {
let kind = match kind { PadKind::X360 => DeviceKind::X360, PadKind::Ds4 => DeviceKind::DS4, PadKind::Ds5 => DeviceKind::DS5 };
let h = spawn(kind).await?;
let _ = set_rumble(h, 0, 0).await?;
}
}
Ok(())
}
