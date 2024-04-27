# Ambient app

Creating an app for the Ambient runtime host.

RUST_LOG=INFO nohup cargo run --example hello_world --target=aarch64-apple-darwin

RUST_LOG=INFO RUST_BACKTRACE=1 cargo run --example hello_world --target=aarch64-apple-darwin