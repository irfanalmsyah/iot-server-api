use std::sync::Arc;

use ntex_mqtt::{v3, v5};

use crate::{database::PgConnection, models::feeds::FeedPayload, utils::auth::verify_jwt};

#[derive(Debug)]
pub struct ServerError;

impl From<()> for ServerError {
    fn from(_: ()) -> Self {
        ServerError
    }
}

impl std::convert::TryFrom<ServerError> for ntex_mqtt::v5::PublishAck {
    type Error = ServerError;

    fn try_from(err: ServerError) -> Result<Self, Self::Error> {
        Err(err)
    }
}

#[derive(Clone, Debug)]
pub struct MySession {
    pub user_id: i32,
}

pub async fn handle_handshake_v3(
    handshake: v3::Handshake,
) -> Result<v3::HandshakeAck<MySession>, ServerError> {
    let packet = handshake.packet();
    let username = packet.username.clone().unwrap();
    let jwt_token = username.as_str();
    match verify_jwt(jwt_token).await {
        Ok(claims) => Ok(handshake.ack(
            MySession {
                user_id: claims.user_id,
            },
            false,
        )),
        Err(_) => Err(ServerError),
    }
}

pub async fn handle_publish_v3(
    session: v3::Session<MySession>,
    publish: v3::Publish,
    pg_connection: Arc<PgConnection>,
) -> Result<(), ServerError> {
    let topic = publish.topic();
    let payload = publish.payload();
    if topic.path() == "channel" {
        let data = std::str::from_utf8(payload).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();
        let _ = pg_connection
            .add_feed_from_mqtt(data, session.state().user_id)
            .await;
    }
    Ok(())
}

pub async fn handle_handshake_v5(
    handshake: v5::Handshake,
) -> Result<v5::HandshakeAck<MySession>, ServerError> {
    let packet = handshake.packet();
    let username = packet.username.clone().unwrap();
    let jwt_token = username.as_str();
    match verify_jwt(jwt_token).await {
        Ok(claims) => Ok(handshake.ack(MySession {
            user_id: claims.user_id,
        })),
        Err(_) => {
            println!("JWT verification failed.");
            Err(ServerError)
        }
    }
}

pub async fn handle_publish_v5(
    session: v5::Session<MySession>,
    publish: v5::Publish,
    pg_connection: Arc<PgConnection>,
) -> Result<v5::PublishAck, ServerError> {
    let topic = publish.topic();
    let payload = publish.payload();
    if topic.path() == "channel" {
        let data = std::str::from_utf8(payload).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();
        let _ = pg_connection
            .add_feed_from_mqtt(data, session.state().user_id)
            .await;
    }
    Ok(publish.ack())
}
