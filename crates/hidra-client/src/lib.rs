#![deny(warnings)]

use anyhow::{Result, bail};
use hidra_ipc::{BrokerRequest, BrokerResponse, connect_client, read_json, write_json};
use hidra_protocol::DeviceKind;

#[derive(Debug, Clone, Copy)]
pub struct GamepadHandle(u64);

pub async fn spawn(kind: DeviceKind) -> Result<GamepadHandle> {
    let features = 0u32;
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::Create { kind, features }).await?;
    match read_json::<BrokerResponse, _>(&mut pipe).await? {
        BrokerResponse::OkCreate { handle } => Ok(GamepadHandle(handle)),
        BrokerResponse::Err { message } => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}

pub async fn set_rumble(_h: GamepadHandle, _low: u8, _high: u8) -> Result<()> {
    Ok(())
}

pub async fn destroy(h: GamepadHandle) -> Result<()> {
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::Destroy { handle: h.0 }).await?;
    match read_json::<BrokerResponse, _>(&mut pipe).await? {
        BrokerResponse::Ok => Ok(()),
        BrokerResponse::Err { message } => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}
