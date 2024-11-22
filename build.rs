use dotenvy::dotenv;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in the .env file");

    let byte_array = jwt_secret
        .as_bytes()
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("jwt_secret.rs");
    fs::write(
        dest_path,
        format!("pub const JWT_SECRET: &[u8] = &[{}];", byte_array),
    )
    .unwrap();
}
