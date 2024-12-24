use std::borrow::Cow;

use sonic_rs::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Hardware {
    pub id: i32,
    pub name: Cow<'static, str>,
    pub type_: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

#[derive(Serialize, Deserialize)]
pub struct HardwarePayload {
    pub name: Cow<'static, str>,
    pub type_: Cow<'static, str>,
    pub description: Cow<'static, str>,
}
