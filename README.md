# mintaka
(WIP) A high-performance PVS-based Renju engine with strict rule implementation.

## Features

- **Core Engine**
  - Full [Renju-rule](https://www.renju.net/rules/) implementation featuring strict forbidden move detection:
    - Accurate detection of single-line and nested forbidden moves (overlines, double-threes, double-fours)
    - See [detailed examples](./documents/renju.md) for complete specifications
  - Advanced tree search algorithm based on Principal Variation Search (PVS) with dedicated VCF solver
  - [Lazy-SMP](https://en.wikipedia.org/wiki/Lazy_SMP) parallel processing
  - SIMD acceleration (AVX-512, AVX2, SSE2)

- **Interfaces**
  - Protocol: [Gomocup (pbrain)](https://plastovicka.github.io/protocl2en.htm), [GUI (Yixin-board)](https://github.com/accreator/Yixin-Board)
  - Language Bindings: Java (JNI), WebAssembly/JavaScript (WIP)
  - Web Frontend: Solid.js Web UI and REST API server (WIP)

- **Planned**
  - [NNUE](https://www.chessprogramming.org/NNUE) based evaluation with separate black/white networks
  - Big-endian system support

## Project Structure

### rusty-renju
Core Renju rule implementation

### mintaka
PVS-based search engine core

### mintaka-interface
Protocol implementations ([Gomocup](https://plastovicka.github.io/protocl2en.htm), [Yixin-board](https://github.com/accreator/Yixin-Board), CLI)

### mintaka-server
REST API backend

### mintaka-webui
Solid.js frontend

### mintaka-js
TypeScript/WebAssembly bindings

### rusty-renju-jni
Java Native Interface bindings
