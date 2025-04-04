# mintaka
(WIP) A high-performance PVS-based Renju engine with strict rule implementation.

## Features

- **Core Engine**
  - Full [Renju-rule](https://www.renju.net/rules/) implementation featuring [strict forbidden move](./documents/renju.md) detection.
  - Advanced tree search algorithm based on [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search) with dedicated VCF solver
  - [Lazy-SMP](https://en.wikipedia.org/wiki/Lazy_SMP) parallel processing for multithreaded search
  - Hardware acceleration with SIMD (AVX-512, AVX2, SSE2, Neon) and bit-manipulation (BMI2) instructions

- **Interfaces**
  - Protocol Support: [Piskvork(Gomocup)](https://plastovicka.github.io/protocl2en.htm), [Yixin-board](https://github.com/accreator/Yixin-Board) and [GTP](https://www.gnu.org/software/gnugo/gnugo_19.html)-like protocol.
  - FFI Bindings: Java (JNI), WebAssembly (JavaScript/TypeScript)
  - Web Interface: Solid.js frontend with REST API backend

- **Planned**
  - [NNUE](https://en.wikipedia.org/wiki/Efficiently_updatable_neural_network)-based evaluation with separate networks for black/white pieces
  - Opening book support for early game optimization

## Project Structure

### rusty-renju
Renju rule implementation with strict forbidden move detection

### mintaka
Principal Variation Search (PVS) based engine core

### mintaka-interface
Protocol adapters for [Piskvork(Gomocup)](https://plastovicka.github.io/protocl2en.htm), [Yixin-board](https://github.com/accreator/Yixin-Board) and [GTP](https://www.gnu.org/software/gnugo/gnugo_19.html)-like CLI

### mintaka-server
(TBD) REST API service for web integration

### mintaka-webui
(TBD) Solid.js based web frontend

### mintaka-wasm
(TBD) WebAssembly bindings for JavaScript/TypeScript

### rusty-renju-jni
Java Native Interface (JNI) bindings

### rusty-renju-image
Utility library for generating board visualizations

### mintaka-trainer
(WIP) Machine learning pipeline for NNUE training
