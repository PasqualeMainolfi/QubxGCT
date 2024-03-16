# Grain Cloude Texture GCT

Granular synthesis in rust using Qubx (see <https://github.com/PasqualeMainolfi/Qubx>)  

## Usage

Clone and compile qubx

```shell
git clone --depth 1 https://github.com/PasqualeMainolfi/Qubx
cd Qubx/qubx
cargo build --release
```

Add Qubx to dependencies. In QubxGCT Cargo.toml file change the path to qubx lib

```code
[dependencies]
qubx = { path = "path_to/qubx" }
```

compile and run

```shell
cargo build --release
cargo run --release
```
