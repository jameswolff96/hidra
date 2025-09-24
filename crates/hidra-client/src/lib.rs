#![deny(warnings)]

use anyhow::{Result, bail};
use hidra_ipc::{BrokerRequest, BrokerResponse, PadState, connect_client, read_json, write_json};
use hidra_protocol::DeviceKind;

#[derive(Debug, Clone, Copy)]
pub struct GamepadHandle(pub u64);

pub async fn ping() -> Result<()> {
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::Ping).await?;
    match read_json::<BrokerResponse, _>(&mut pipe).await? {
        BrokerResponse::Pong => Ok(()),
        BrokerResponse::Err { message } => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}

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

pub async fn update_state(h: GamepadHandle, s: PadState) -> Result<()> {
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::UpdateState { handle: h.0, state: s }).await?;
    match read_json::<BrokerResponse, _>(&mut pipe).await? {
        BrokerResponse::Ok => Ok(()),
        BrokerResponse::Err { message } => bail!("broker error: {message}"),
        other => bail!("unexpected response: {:?}", other),
    }
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
