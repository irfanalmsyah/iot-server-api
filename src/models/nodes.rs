use std::borrow::Cow;

use sonic_rs::{Deserialize, Serialize};

use super::feeds::Feed;

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
    pub user_id: i32,
    pub hardware_id: i32,
    pub name: Cow<'static, str>,
    pub location: Cow<'static, str>,
    pub hardware_sensor_ids: Vec<i32>,
    pub hardware_sensor_names: Vec<Cow<'static, str>>,
    pub ispublic: bool,
}

#[derive(Serialize, Deserialize)]
pub struct NodeWithFeed {
    pub node: Node,
    pub feeds: Vec<Feed>,
}
