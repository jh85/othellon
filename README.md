# Othellon

An Othello (Reversi) game engine supporting board sizes from 4x4 to 20x20. Play in the browser with a web UI, or use the GTP protocol to connect to tools like GoGui.

## Features

- **Variable board sizes**: Even sizes from 4x4 to 20x20
- **AI opponent**: 5 difficulty levels (random, shallow, default, deep, deeper) using alpha-beta search with bitboard move generation
- **Web UI**: Phaser 3 based interface with legal move hints, score graph, game history, and SGF export
- **GTP support**: Compatible with GoGui and other GTP-capable frontends
- **Game modes**: Human vs Human, Human vs AI, AI vs AI with auto-play

## Prerequisites

- [Rust](https://rustup.rs/) toolchain
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- Python 3

## Build

```sh
wasm-pack build crates/othello-wasm --target web --out-dir ../../pkg
cargo build --release
```

## Run

```sh
python3 server.py
```

Open http://localhost:8080 in a browser.

## GTP

The GTP engine binary can be used with GoGui or any GTP-compatible frontend:

```sh
./target/release/othello-gtp
```

## Project Structure

```
crates/
  othello-core/   # Game logic, bitboard, AI search
  othello-gtp/    # GTP protocol engine
  othello-wasm/   # WASM bindings for the web UI
web/              # Browser frontend (Phaser 3)
server.py         # HTTP server with game history API
```
