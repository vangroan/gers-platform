
⚠ PROTOTYPE

# GERS Platform

Experimental game engine / framework that's scriptable using [WebAssembly](https://github.com/WebAssembly).

## Build

This project requires [`cargo-make`](https://github.com/sagiegurari/cargo-make) to build, because it mixes native targets with WASM targets. The build process requires some more involved tasks that can't be covered with `cargo build` alone.

```shell
# Install make
cargo install cargo-make
# Run build and tests
cargo make
```

## Goals

- Modding - It should be trivial to extend the functionality of game.
- Deterministic Simulation - To implement lockstep networking, execution of the same game logic on different machines must result in the same program state.

## State of the Art

- `wasm-bindgen` is for Rust WebAssembly in Javascript (V8). How shims and bindings for Rust WebAssembly in Rust (`wasmer`, `wasm-time`) would work are unclear.
  - https://github.com/wasmerio/wasmer/issues/315
  - https://github.com/wasmerio/wasmer/issues/553
- `panic!()` or `format!()` not optimised away adds enourmous bloat to the generated WebAssembly. At the moment there is little that can be done about it.
  - https://github.com/rustwasm/team/issues/19
  - https://github.com/rust-embedded/wg/issues/41
- Cargo workspaces cannot have distinct build targets per crate.
  - https://github.com/rust-lang/cargo/issues/7004
- WASM as a Platform for Abstraction
  - https://adventures.michaelfbryan.com/posts/wasm-as-a-platform-for-abstraction/
  - https://users.rust-lang.org/t/wasm-as-a-platform-for-abstraction/35736
- Amethyst Scripting RFC
  - https://github.com/amethyst/rfcs/pull/1
- WASM interpreter in C# for games
  - https://github.com/rockyjvec/GameWasm

## Licence

gers-platform is distributed under the MIT licence.

See [LICENCE](LICENCE) for details.

