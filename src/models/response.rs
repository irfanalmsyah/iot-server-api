use sonic_rs::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<'a, T> {
    pub message: &'a str,
    pub data: Vec<T>,
}
