cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-name wasm_particular-demo --out-dir docs --target web target/wasm32-unknown-unknown/release/bevy-particular-demo.wasm 
