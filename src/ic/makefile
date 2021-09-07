all:
	cargo build --release --target wasm32-unknown-unknown
	ic-cdk-optimizer target/wasm32-unknown-unknown/release/dfnhack7.wasm -o target/wasm32-unknown-unknown/release/dfnhack7-opt.wasm
