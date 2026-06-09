# ternary-channel

**Communication channel abstractions for inter-room messaging**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-25-green)]()

## Overview

Communication channel abstractions for inter-room messaging.

Provides the comms backbone connecting rooms in a ternary fleet:
direct channels, broadcast, priority ordering, reliable delivery with
acknowledgment and retry, and multiplexing many logical channels over
one connection.

## Architecture

- **`Message`** — core data structure
- **`DirectChannel`** — core data structure
- **`BroadcastChannel`** — core data structure
- **`PriorityChannel`** — core data structure
- **`ReliableChannel`** — core data structure
- **`ChannelMux`** — core data structure
- **`TernaryPriority`** — state enumeration
- **`ChannelState`** — state enumeration
- **`DeliveryStatus`** — state enumeration

### Traits

- **`Channel`** — shared behavior contract

### Key Functions

- `new()`
- `new()`
- `name()`
- `new()`
- `name()`
- `subscribe()`
- `unsubscribe()`
- `subscriber_count()`
- `subscribers()`
- `new()`
- ... and 19 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 741 |
| Test count | 25 |
| Public types | 9 |
| Public functions | 29 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-channel = "0.1.0"
```

```rust
use ternary_channel;
```

## License

MIT
