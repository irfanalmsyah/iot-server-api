# IoT Server API with Rust Ntex

# commands
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh #install rust
sudo apt install postgresql libpq-dev #install postgres
sudo service postgresql start #run postgres
cargo install diesel_cli --no-default-features --features postgres #install diesel postgres
RUSTFLAGS="-C target-cpu=native" cargo build --release #build
./target/release/iot-server-api #run
```