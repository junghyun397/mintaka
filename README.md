# mintaka
(WIP) A high-performance PVS-based Renju engine with strict rule implementation.

## Features

- **Core Engine**
  - Full [Renju-rule](https://www.renju.net/rules/) implementation featuring strict forbidden move detection:
    - Accurate detection of single-line and nested forbidden moves (overlines, double-threes, double-fours)
    - See [detailed examples](./documents/renju.md) for complete specifications
  - Advanced tree search algorithm based on [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search) (PVS) with dedicated VCF solver
  - [Lazy-SMP](https://en.wikipedia.org/wiki/Lazy_SMP) parallel processing for multithreaded search
  - Hardware acceleration with SIMD instructions (AVX-512, AVX2, SSE2, Neon) and bit-manipulation instructions (BMI2)

- **Interfaces**
  - Protocol Support: [Piskvork(Gomocup)](https://plastovicka.github.io/protocl2en.htm), [Yixin-board](https://github.com/accreator/Yixin-Board)
  - FFI Bindings: Java (JNI), WebAssembly (JavaScript/TypeScript)
  - Web Interface: Solid.js frontend with REST API backend

- **Planned**
  - [NNUE](https://en.wikipedia.org/wiki/Efficiently_updatable_neural_network)-based evaluation with separate networks for black/white pieces
  - Opening book support for early game optimization

## Project Structure

### rusty-renju
Core Renju rule implementation

### mintaka
Principal Variation Search (PVS) based engine core

### mintaka-interface
Protocol adapters for [Gomocup](https://plastovicka.github.io/protocl2en.htm), [Yixin-board](https://github.com/accreator/Yixin-Board) and CLI

### mintaka-server
REST API service for web integration

### mintaka-webui
Solid.js based web frontend

### mintaka-wasm
WebAssembly bindings for JavaScript/TypeScript

### rusty-renju-jni
Java Native Interface (JNI) bindings

### mintaka-trainer
Machine learning pipeline for NNUE training
