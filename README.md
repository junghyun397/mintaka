# mintaka
(WIP) HCE-based Renju engine with strict Renju-rule implementation.

* Strict [Renju-rule](https://www.renju.net/rules/) support w/ [single line/nested forbidden moves](https://www.renju.net/advanced/).
* AVX-512(512-bits), AVX2(256-bits), Neon(128-bits) SIMD-accelerated.
* (WIP) High-performance endgame VCT/VCF calculator.
* (WIP) [Negmax](https://en.wikipedia.org/wiki/Negamax) w/ HCE(Hand-Crafted Evaluation) based strong A.I.
* (WIP) [Solid.js](https://www.solidjs.com/)-based Web Frontend UI and REST API server.
* (WIP) Java Native Interface([JNI](https://en.wikipedia.org/wiki/Java_Native_Interface)) support.
* (TBD) WebAssembly and JavaScript Interface support.
* (TBD) Efficiently updatable neural network-based evaluation function([NNUE](https://www.chessprogramming.org/NNUE)).
* (TBD) Big-endian system compatibility.

## mintaka
renju-board implementation.

## mintaka-engine
HCE-based renju engine.

## mintaka-server
REST API backend server for web-ui frontend.

## mintaka-webui
Web-frontends built with Solid.js.
