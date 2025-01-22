#[cfg(not(target_os = "macos"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod app;
mod constant;
mod database;
mod handlers;
mod models;
mod mqtt;
mod secure;
mod unsecure;
mod utils;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "secure" {
        secure::run_secure().await
    } else {
        unsecure::run_unsecure().await
    }
}
