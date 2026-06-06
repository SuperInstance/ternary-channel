# ternary-channel

**Communication channel abstractions for inter-room messaging in the SuperInstance ternary ecosystem.**

## Background

In distributed systems, the communication substrate determines everything else — fault tolerance, throughput, ordering guarantees, and latency characteristics. Traditional message-passing systems (like Erlang/OTP actors, Go channels, or MPI) use binary channel states: a message is either sent or not sent, a channel is open or closed. The SuperInstance ecosystem introduces a **ternary paradigm** where every signal carries three possible semantic values — negative (−1), neutral (0), or positive (+1) — enabling richer communication semantics than binary on/off.

`ternary-channel` provides the comms backbone connecting "rooms" in a ternary fleet. It offers five channel types, each with bounded capacity, clean lifecycle management, and ternary priority semantics for message ordering.

## How It Works

### Core Abstractions

The crate defines a `Channel` trait as the universal interface:

- **`send(msg) → Result`** — enqueue a message
- **`receive() → Option<Message>`** — dequeue (non-blocking)
- **`close()`** — terminate the channel
- **`is_open() → bool`** — check liveness
- **`pending_count() → usize`** — queue depth

Every message carries a `TernaryPriority`: `Negative`, `Neutral`, or `Positive`, mapped to −1, 0, and +1 respectively. This priority drives ordering in priority-aware channel types.

### Channel Types

| Channel | Purpose | Ordering |
|---------|---------|----------|
| `DirectChannel` | Point-to-point between two endpoints | FIFO |
| `BroadcastChannel` | One-to-many with pub/sub semantics | FIFO, fan-out |
| `PriorityChannel` | Ternary-priority-ordered delivery | Positive → Neutral → Negative |
| `ReliableChannel` | ACK-based delivery with retries | FIFO + tracking |
| `ChannelMux` | Multiplex many logical channels over one connection | Per-sub-channel FIFO |

### Reliable Delivery

`ReliableChannel` implements acknowledgment tracking: every sent message has a `DeliveryStatus` (Pending → Acknowledged or Failed). The `retry_pending()` method advances retry counters, and `clean_outbox()` prunes completed messages. This mirrors patterns from TCP retransmission and MQTT QoS 2.

### Multiplexing

`ChannelMux` multiplexes named `DirectChannel` instances over a single connection, supporting `send_on(channel_name, msg)`, `receive_from(channel_name)`, and `receive_any()` for round-robin dispatch. This is structurally similar to HTTP/2 stream multiplexing or SSH channel multiplexing.

## Experimental Results

The crate includes comprehensive test coverage (30+ unit tests) validating:

- **FIFO ordering** in direct and broadcast channels
- **Priority dequeue** always drains positive-priority messages before neutral, and neutral before negative
- **Capacity enforcement** — bounded buffers reject overflow
- **Close semantics** — closed channels reject sends, existing messages remain receivable
- **Retry lifecycle** — messages transition from Pending → Acknowledged (on `ack()`) or Pending → Failed (after max retries)
- **Mux isolation** — sub-channels operate independently within the multiplexer

## Impact

`ternary-channel` serves as the foundational transport layer for the SuperInstance ecosystem. By embedding ternary priority directly into the messaging substrate, higher-level protocols (consensus, voting, game theory) can leverage priority-aware routing without additional infrastructure. The design follows the Unix philosophy: simple primitives that compose into powerful systems.

The multiplexer pattern enables efficient resource usage in fleet deployments where many logical conversations share a single network connection, while the reliable channel provides the building blocks for exactly-once delivery semantics.

## Use Cases

1. **Inter-room task dispatch** — A fleet coordinator sends high-priority (Positive) task assignments to worker rooms, while low-priority telemetry (Negative) is queued behind urgent messages. `PriorityChannel` ensures critical commands are never blocked by bulk data.

2. **Reliable command delivery** — Control messages between agents use `ReliableChannel` for acknowledgment tracking and automatic retry. Failed deliveries surface as `DeliveryStatus::Failed` for upstream error handling, similar to MQTT's QoS guarantees.

3. **Multi-protocol multiplexing** — A single TCP connection between two fleet nodes carries command traffic, event notifications, and heartbeat signals on separate logical channels via `ChannelMux`, avoiding head-of-line blocking between traffic classes.

4. **Broadcast coordination** — `BroadcastChannel` distributes configuration updates to all subscribed rooms simultaneously, with subscriber management (subscribe/unsubscribe) for dynamic fleet membership.

5. **Telemetry aggregation** — Low-priority metric reports flow through `DirectChannel` instances from leaf rooms to a central collector, respecting capacity limits to prevent backpressure from overwhelming the collector.

## Open Questions

- **Backpressure propagation:** The current bounded-buffer approach rejects messages at capacity. Should a future version support backpressure signals (e.g., `await_capacity()`) for flow control, similar to reactive streams?
- **Asynchronous I/O:** The current design is synchronous (non-blocking `receive`). Integration with `tokio` or `async-std` would enable async `send`/`receive` for high-concurrency fleet deployments. What's the right abstraction boundary?
- **Persistence:** Channels are in-memory only. For fleet survivability across node restarts, should channel state be persistable (WAL, shared memory)?

## Connection to Oxide Stack

`ternary-channel` is the transport backbone of the SuperInstance ternary ecosystem. It is consumed by:

- **`ternary-event`** — event bus uses channels internally for dispatch
- **`ternary-command`** — command dispatch uses reliable channels for ACK-based command delivery
- **`ternary-protocol`** — wire protocol defines serialization for channel messages
- **`ternary-voting`** — consensus rounds use priority channels for vote collection

The channel abstractions follow the same philosophical pattern as Rust's `std::sync::mpsc` and Go's channel model, but with ternary priority semantics woven into the core trait — a design choice that propagates through the entire ecosystem.
