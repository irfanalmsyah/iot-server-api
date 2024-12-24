use ntex::http::header::{CONTENT_TYPE, SERVER};
use ntex::{
    http::{Response, StatusCode},
    util::{Bytes, BytesMut},
};

use sonic_rs::{to_writer, Serialize};

use crate::models::response::ApiResponse;

use super::{reserve, BytesWriter, HDR_JSON_CONTENT_TYPE, HDR_SERVER};

pub fn serialize_response<T: Serialize>(
    response: ApiResponse<T>,
    status: StatusCode,
) -> (Bytes, StatusCode) {
    let mut body = BytesMut::with_capacity(10 * 1024 * 1024);
    reserve(&mut body, 10 * 1024 * 1024);
    to_writer(BytesWriter(&mut body), &response).unwrap();

    (body.split().freeze(), status)
}

pub fn response_json(data: Bytes, status: StatusCode) -> Response {
    let mut res = Response::with_body(status, data.into());
    res.headers_mut()
        .insert(CONTENT_TYPE, HDR_JSON_CONTENT_TYPE);
    res.headers_mut().insert(SERVER, HDR_SERVER);

    res
}

pub fn extract_id_from_path(path: &str, prefix: &str) -> Option<i32> {
    path.strip_prefix(prefix)
        .and_then(|p| p.strip_suffix("/"))
        .and_then(|id_str| id_str.parse::<i32>().ok())
}
