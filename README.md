# mintaka
(WIP) PVS-based Renju engine with strict Renju-rule implementation.

* Strict [Renju-rule](https://www.renju.net/rules/) support w/ [single line/nested forbidden moves](./documents/renju.md).
* [Principal variation search](https://en.wikipedia.org/wiki/Principal_variation_search) based strong A.I.
* High-performance endgame VCF/VCT calculator.
* [Lazy-SMP](https://en.wikipedia.org/wiki/Lazy_SMP) based multicore processing.
* AVX-512(512-bits), AVX2(256-bits) SIMD-accelerated.
* Java Native Interface(JNI) support.
* (WIP) [Solid.js](https://www.solidjs.com/)-based Web Frontend UI and REST API server.
* (TBD) WebAssembly and JavaScript Interface support.
* (TBD) Efficiently updatable neural network-based evaluation function([NNUE](https://www.chessprogramming.org/NNUE)).
* (TBD) Big-endian system compatibility.

## rusty-renju
Renju-Board Implementation.
 * ``strict-renju``: Enable nesting double-three checks.

## rusty-renju-jni
Java Native Interface Implementation for rusty-renju.

## rusty-renju-ecma
TypeScript/Webassembly Interface Implementation for rusty-renju.

## mintaka
PVS+HCE based renju engine.

## mintaka-server
REST API backend server for web-ui frontend.

## mintaka-webui
Web-frontends built with Solid.js.
