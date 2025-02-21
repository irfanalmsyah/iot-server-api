use crate::{
    constant::config,
    database,
    models::{feeds::FeedPayload, jwt::Claims},
};
use deadpool_postgres::Pool;
use jsonwebtoken::DecodingKey;
use ntex_mqtt::{v3, v5};

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
    let token = username.as_str();
    match jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config::NODE_JWT_SECRET.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => Ok(handshake.ack(
            MySession {
                user_id: token_data.claims.user_id,
            },
            false,
        )),
        Err(_) => Err(ServerError),
    }
}

pub async fn handle_publish_v3(
    session: v3::Session<MySession>,
    publish: v3::Publish,
    pool: Pool,
) -> Result<(), ServerError> {
    let topic = publish.topic();
    let payload = publish.payload();
    if topic.path() == "channel" {
        let data = std::str::from_utf8(payload).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();
        let client = pool.get().await.unwrap();
        let _ = database::feeds::add_feed_from_mqtt(&client, data, session.state().user_id).await;
    }
    Ok(())
}

pub async fn handle_handshake_v5(
    handshake: v5::Handshake,
) -> Result<v5::HandshakeAck<MySession>, ServerError> {
    let packet = handshake.packet();
    let username = packet.username.clone().unwrap();
    let token = username.as_str();
    match jsonwebtoken::decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config::NODE_JWT_SECRET.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => Ok(handshake.ack(MySession {
            user_id: token_data.claims.user_id,
        })),
        Err(_) => Err(ServerError),
    }
}

pub async fn handle_publish_v5(
    session: v5::Session<MySession>,
    publish: v5::Publish,
    pool: Pool,
) -> Result<v5::PublishAck, ServerError> {
    let topic = publish.topic();
    let payload = publish.payload();
    if topic.path() == "channel" {
        let data = std::str::from_utf8(payload).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();
        let client = pool.get().await.unwrap();
        let _ = database::feeds::add_feed_from_mqtt(&client, data, session.state().user_id).await;
    }
    Ok(publish.ack())
}
