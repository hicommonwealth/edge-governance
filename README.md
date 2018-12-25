# edge_governance
This module contains the logic that powers Edgeware's governance UI. It is presented as a broader governance module that forms something
akin to a forum for governance proposals. Users can submit proposals, vote on proposals, and track progress of proposals through Edgeware's
governance process.

# Setup
Install rust or update to the latest versions.
```
curl https://sh.rustup.rs -sSf | sh
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```

You will also need to install the following packages:

Linux:
```
sudo apt install cmake pkg-config libssl-dev git
```

Mac:
```
brew install cmake pkg-config openssl git
```
