# ternary-channel — Communication channel abstractions for inter-room messaging

The comms backbone that connects rooms in a ternary fleet. Direct channels, broadcast, priority ordering, reliable delivery with acknowledgment and retry, and multiplexing.

## Why This Exists

Agents in different rooms need to talk to each other. Not all messages are equal — some are urgent (positive priority), some are routine (neutral), and some can wait (negative). Not all channels have the same reliability requirements. This crate provides typed channel abstractions that let you pick the right semantics for each communication pattern.

## Core Concepts

- **Channel trait** — The base interface: `send`, `receive`, `close`, `is_open`, `pending_count`. All channel types implement this.
- **DirectChannel** — Point-to-point FIFO buffer between two endpoints. Simple and predictable.
- **BroadcastChannel** — One-to-many. Messages go to all subscribers. Subscribers are tracked separately from the message buffer.
- **PriorityChannel** — Messages ordered by ternary priority. Positive messages are dequeued first, then neutral, then negative. Within each tier, FIFO order applies.
- **ReliableChannel** — Adds acknowledgment tracking and retry logic. Messages live in an outbox until acknowledged. Failed messages (max retries exceeded) are marked accordingly.
- **ChannelMux** — Multiplexes many named logical channels over one connection. Route messages by channel name; receive from specific channels or any channel.
- **TernaryPriority** — Three priority levels: Positive (high), Neutral (normal), Negative (low). Maps directly to the ternary value system.

## Quick Start

```toml
[dependencies]
ternary-channel = "0.1"
```

```rust
use ternary_channel::*;

// Priority channel: urgent messages first
let mut pq = PriorityChannel::new("control", 100);
pq.send(Message::new(1, "room-a", "room-b", vec![1], TernaryPriority::Negative)).unwrap();
pq.send(Message::new(2, "room-a", "room-b", vec![2], TernaryPriority::Positive)).unwrap();
assert_eq!(pq.receive().unwrap().id, 2); // positive first

// Reliable channel with retries
let mut rel = ReliableChannel::new("critical", 50, 3);
rel.send(Message::new(1, "a", "b", vec![], TernaryPriority::Positive)).unwrap();
rel.ack(1).unwrap(); // confirm delivery

// Multiplex several channels
let mut mux = ChannelMux::new();
mux.add_channel("heartbeat", 50).unwrap();
mux.add_channel("data", 200).unwrap();
mux.send_on("data", Message::new(1, "a", "b", vec![42], TernaryPriority::Neutral)).unwrap();
```

## API Overview

| Type | Description |
|------|-------------|
| `Channel` (trait) | Base interface for send/receive/close |
| `DirectChannel` | Point-to-point FIFO with capacity limit |
| `BroadcastChannel` | One-to-many with subscriber tracking |
| `PriorityChannel` | Ternary-ordered dequeue (positive first) |
| `ReliableChannel` | Acknowledgment tracking with retry logic |
| `ChannelMux` | Named channel multiplexer |
| `Message` | Payload with ID, sender, recipient, priority |
| `TernaryPriority` | Positive, Neutral, Negative ordering |
| `DeliveryStatus` | Pending, Acknowledged, Failed |

## How It Works

All channel types share the same `Channel` trait. Internally, they use `VecDeque` for buffering. Capacity is bounded — `send` returns an error when full.

**PriorityChannel** maintains three separate queues (one per priority level). On `receive`, it checks positive first, then neutral, then negative. This is O(1) per dequeue.

**ReliableChannel** tracks sent messages in an `outbox` with retry counts. `retry_pending()` increments the retry counter for all pending messages and marks those past `max_retries` as Failed. `ack()` marks a message as Acknowledged. `clean_outbox()` removes completed entries.

**ChannelMux** holds a vector of named `DirectChannel` instances. `send_on` and `receive_from` target specific channels by name. `receive_any` scans channels in order and returns the first available message.

## Known Limitations

- No async support. This is a synchronous, in-process channel library. For network transport, wrap in your own async layer.
- BroadcastChannel delivers messages from a single buffer — subscribers don't get individual copies. The application is responsible for delivering to each subscriber.
- ReliableChannel doesn't actually re-send; it just tracks retry state. The caller must implement actual retransmission logic.
- No message ordering guarantees across priorities in DirectChannel (it's pure FIFO). Use PriorityChannel for ordering.
- ChannelMux uses linear search by name. Fine for dozens of channels; not ideal for thousands.

## Use Cases

- **Control plane** — PriorityChannel for fleet commands where some directives override others.
- **Event bus** — BroadcastChannel for publishing fleet-wide events (agent joined, room created).
- **Critical messaging** — ReliableChannel for messages that must be acknowledged (configuration changes, state mutations).
- **Connection sharing** — ChannelMux to run multiple logical channels over a single network connection.

## Ecosystem Context

Part of the SuperInstance ternary fleet ecosystem. The communication layer that connects `ternary-room` instances, carries `ternary-beacon` discovery messages, and coordinates `ternary-harbor` docking operations. This is the plumbing; other crates are the fixtures.

## License

MIT
