# mintaka_wasm

WebAssembly (wasm-bindgen) bindings for `mintaka` (Renju engine) and related types.

* `notation`::`Pos`
* `notation`::`Color`

* `rusty_renju`::`Board`

* `mintaka`::`SearchObjective`
* `mintaka`::`GameState`
* `mintaka`::`GameAgent`

## Build

```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

```shell
wasm-pack build mintaka_wasm \
  --release \
  --target web \
  --out-name mintaka_wasm \
  -- -Z build-std=std,panic_abort
```

## Before Run
mintaka-wasm requires SharedArrayBuffer to be enabled for multithreading. So, COOP/COEP must be enabled.

In `vite.config.js`:
```javascript
import { defineConfig } from 'vite'

export default defineConfig({
    server: {
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
        },
    },
})
```

In Cloudflare Pages `headers_`:
```text
/*
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp
```

## Run

### Basic Types
`main.ts`:
```ts
import init, { GameState, Pos } from "./pkg/mintaka_wasm.js";

await init(new URL("./pkg/mintaka_wasm_bg.wasm", import.meta.url));

const state = new GameState();
state.playMut(Pos.fromString("h8"));
state.playMut(Pos.fromString("h9"));
console.log(state.len(), state.board().toString());
```

### Launch Agent
`GameAgent.launch()` is intended to run inside a dedicated worker (it posts search responses via `postMessage`).

`engine.worker.ts`:
```ts
```

`main.ts`:
```ts
```
