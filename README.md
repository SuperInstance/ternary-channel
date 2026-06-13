# Ternary Channel

**Ternary Channel** provides communication abstractions for inter-room messaging — direct channels, broadcast, priority-ordered delivery, reliable transport with acknowledgment and retry, and multiplexing many logical channels over one connection.

## Why It Matters

Reliable communication between distributed rooms requires more than TCP: messages need priorities (urgent vs. normal), multiplexing (multiple logical streams over one connection), and reliability guarantees (at-least-once delivery with retry). Ternary Channel provides these with ternary priority levels {-1 (low), 0 (normal), +1 (high)}, enabling precedence-based message ordering that integrates with the ternary ecosystem.

## How It Works

### Channel Trait

```rust
trait Channel {
    fn send(&mut self, msg: Message) -> Result<(), &'static str>;
    fn try_recv(&mut self) -> Option<Message>;
    fn close(&mut self);
    fn state(&self) -> ChannelState;
}
```

All implementations follow this trait. Send: **O(1)** amortized. Recv: **O(1)**.

### Message Priority

```rust
enum TernaryPriority {
    Negative = -1,  // Low priority, background tasks
    Neutral = 0,    // Normal priority, default
    Positive = 1,   // High priority, urgent alerts
}
```

Priority queue orders messages: Positive before Neutral before Negative. Within same priority: FIFO.

### Direct Channel

Point-to-point channel with bounded buffer:

```
DirectChannel {
    buffer: VecDeque<Message>,  // capacity-bounded
    state: Open | Closed,
}

send(msg):
    if buffer full: return Err("channel full")
    insert by priority: O(N) for sorted insert, O(1) for unsorted append
```

### Broadcast Channel

One sender, many receivers:

```
BroadcastChannel {
    receivers: Vec<Receiver>,
}

send(msg):
    for rx in receivers: rx.enqueue(msg.clone())
```

Broadcast: **O(N)** for N receivers. Each receiver has independent bounded queue.

### Reliable Channel

Wraps any channel with acknowledgment and retry:

```
ReliableChannel<C: Channel> {
    inner: C,
    pending: HashMap<msg_id, (Message, retry_count, last_sent)>,
    ack_timeout: Duration,
    max_retries: usize,
}

send(msg):
    inner.send(msg)
    pending.insert(msg.id, (msg, 0, now))

on_ack(msg_id):
    pending.remove(msg_id)

retry_loop():
    for (id, (msg, count, last)) in pending:
        if now - last > ack_timeout:
            if count >= max_retries: return Err
            inner.send(msg)
            count += 1
            last = now
```

Retry: **O(P)** for P pending messages per timeout interval.

### Multiplexer

```
Mux {
    channels: HashMap<channel_id, Box<dyn Channel>>,
}

send(channel_id, msg) → channels[channel_id].send(msg)
```

Routing: **O(1)** HashMap lookup. Enables many virtual streams over one transport.

## Quick Start

```rust
use ternary_channel::{DirectChannel, Message, TernaryPriority, Channel};

let mut tx = DirectChannel::new(256);
let mut rx = tx.clone_for_receive();

tx.send(Message::new(1, "room-a", "room-b", vec![0x42], TernaryPriority::Positive)).unwrap();

if let Some(msg) = rx.try_recv() {
    println!("Priority: {:?}, payload: {:?}", msg.priority, msg.payload);
}
```

## API

| Type | Description |
|------|-------------|
| `Channel` | Core trait: `send`, `try_recv`, `close`, `state` |
| `DirectChannel` | Point-to-point bounded buffer |
| `BroadcastChannel` | Fan-out to multiple receivers |
| `ReliableChannel<C>` | Acknowledgment and retry wrapper |
| `Mux` | Multiplexer for virtual channels |
| `Message` | id, sender, recipient, payload, priority |
| `TernaryPriority` | Negative (-1), Neutral (0), Positive (+1) |

## Architecture Notes

Ternary Channel provides the communication primitives for inter-room messaging in SuperInstance. In γ + η = C, Positive (+1) priority messages drive γ (growth — urgent coordination for expansion), Negative (-1) priority messages carry η (avoidance — background failure reports and cleanup), and the multiplexer ensures both coexist without blocking. Integrates with `ternary-bus` for pub/sub and `superinstance-protocol` for wire format.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for communication architecture.

## References

1. Hoare, C. A. R. (1978). "Communicating Sequential Processes." *Communications of the ACM*, 21(8), 666–677.
2. Tanenbaum, A. S. & Van Steen, M. (2017). *Distributed Systems*, 3rd ed. Pearson.
3. Tokio Documentation (2024). "Asynchronous Channels in Rust."

## License

MIT
