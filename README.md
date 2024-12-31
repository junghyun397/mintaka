# mintaka
(WIP) PVS-based Renju engine with strict Renju-rule implementation.

* Strict [Renju-rule](https://www.renju.net/rules/) support w/ [single line/nested forbidden moves](./documents/renju.md).
* [Principal variation search](https://en.wikipedia.org/wiki/Principal_variation_search) based strong A.I.
* High-performance endgame VCF/VCT calculator.
* AVX-512(512-bits), AVX2(256-bits) SIMD-accelerated.
* Java Native Interface(JNI) support.
* (WIP) [Solid.js](https://www.solidjs.com/)-based Web Frontend UI and REST API server.
* (TBD) WebAssembly and JavaScript Interface support.
* (TBD) Efficiently updatable neural network-based evaluation function([NNUE](https://www.chessprogramming.org/NNUE)).
* (TBD) Big-endian system compatibility.

## mintaka
renju-board implementation.
 * ``strict-renju``: Enable nesting double-three checks.
 * ``slice-prefetch``: Enable prefetch on line-level patterns memoization.
 * ``slice-memo-size``: Size of hash-table for line-level patterns memoization.

## mintaka-engine
PVS+HCE based renju engine.

## mintaka-server
REST API backend server for web-ui frontend.

## mintaka-webui
Web-frontends built with Solid.js.
