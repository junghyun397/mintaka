# mintaka
(WIP) PVS-based Renju engine with strict Renju-rule implementation.

* Strict [Renju-rule](https://www.renju.net/rules/) support w/ [single line/nested forbidden moves](./documents/renju.md).
* [Principal variation search](https://en.wikipedia.org/wiki/Principal_variation_search) based strong search algorithm.
* High-performance endgame VCF calculator.
* [Lazy-SMP](https://en.wikipedia.org/wiki/Lazy_SMP) based multicore processing.
* AVX-512(512-bits), AVX2(256-bits) SIMD-accelerated.
* Java Native Interface(JNI) support.
* (WIP) WebAssembly and JavaScript Interface support.
* (WIP) [Gomocup](http://gomocup.org/)/GUI Protocol support.
* (WIP) [Solid.js](https://www.solidjs.com/)-based Web Frontend UI and REST API server.
* (TBD) Efficiently updatable neural network-based evaluation function([NNUE](https://www.chessprogramming.org/NNUE)).
* (TBD) Seperated NNUE networks for black and white.
* (TBD) Big-endian system compatibility.

## rusty-renju
Renju-Board Implementation.
 * ``strict-renju``: Enable nesting double-three checks.

## rusty-renju-jni
Java Native Interface implementation for rusty-renju.

## mintaka
PVS-based renju engine.

## mintaka-pbrain
Gomocup/GUI Protocol implementation for mintaka.

## mintaka-server
REST API backend server for web-ui frontend.

## mintaka-js
TypeScript/Webassembly Interface implementation for mintaka.

## mintaka-webui
Web-frontends built with Solid.js.
