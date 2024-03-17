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

In the example, by changing the `MODE` variable to `Sound`, the granulator will expect to granulate sampled events. `Synthetic` will generate synthetic grains, and finally `Microphone` for real-time granulation of an event captured from an input device.

```rust
enum GranulatorMode {
    Sound,
    Synthetic,
    Microphone
}

const MODE: GranulatorMode = GranulatorMode::Sound; // samples events
const MODE: GranulatorMode = GranulatorMode::Synthetic; // synthetic events
const MODE: GranulatorMode = GranulatorMode::Microphone; // real-time events from an input device
```

compile and run

```shell
cargo build --release
cargo run --release
```
