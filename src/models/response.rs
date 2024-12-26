use sonic_rs::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<'a, T> {
    pub message: &'a str,
    pub data: Data<T>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Data<T> {
    Single(T),
    Multiple(Vec<T>),
    None,
}
