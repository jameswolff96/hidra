#![deny(warnings)]

use anyhow::{Result, bail};
use hidra_ipc::{
    BrokerRequest, BrokerResponse, PadState, connect_client, read_json_opt, write_json,
};
use hidra_protocol::DeviceKind;
use tracing::{debug, info, instrument};

#[derive(Debug, Clone, Copy)]
pub struct GamepadHandle(pub u64);

#[instrument(level = "debug")]
pub async fn ping() -> Result<()> {
    let mut pipe = connect_client().await?;
    debug!("connected to broker");
    write_json(&mut pipe, &BrokerRequest::Ping).await?;
    match read_json_opt::<BrokerResponse, _>(&mut pipe).await? {
        Some(BrokerResponse::Pong) => {
            info!("broker pong");
            Ok(())
        }
        Some(BrokerResponse::Err { message }) => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}

#[instrument(level = "info", fields(?kind))]
pub async fn spawn(kind: DeviceKind) -> Result<GamepadHandle> {
    let features = 0u32;
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::Create { kind, features }).await?;
    match read_json_opt::<BrokerResponse, _>(&mut pipe).await? {
        Some(BrokerResponse::OkCreate { handle }) => {
            info!(handle, "spawned");
            Ok(GamepadHandle(handle))
        }
        Some(BrokerResponse::Err { message }) => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}

#[instrument(level = "debug", fields(handle=h.0, state=?s))]
pub async fn update_state(h: GamepadHandle, s: PadState) -> Result<()> {
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::UpdateState { handle: h.0, state: s }).await?;
    match read_json_opt::<BrokerResponse, _>(&mut pipe).await? {
        Some(BrokerResponse::Ok) => {
            debug!("updated state");
            Ok(())
        }
        Some(BrokerResponse::Err { message }) => bail!("broker error: {message}"),
        other => bail!("unexpected response: {:?}", other),
    }
}

#[instrument(level = "info", fields(handle=h.0))]
pub async fn destroy(h: GamepadHandle) -> Result<()> {
    let mut pipe = connect_client().await?;
    write_json(&mut pipe, &BrokerRequest::Destroy { handle: h.0 }).await?;
    match read_json_opt::<BrokerResponse, _>(&mut pipe).await? {
        Some(BrokerResponse::Ok) => {
            info!("destroyed");
            Ok(())
        }
        Some(BrokerResponse::Err { message }) => bail!("broker error: {message}"),
        other => bail!("unexpected response from broker: {:?}", other),
    }
}
