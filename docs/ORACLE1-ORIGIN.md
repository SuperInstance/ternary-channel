# Oracle1 Origin: Channel → ternary-channel

## Oracle1 Concept
**Layer 4: Channel** — IRC-like rooms via PLATO server (port 8847, 1,485+ rooms). PLATO provides named rooms where agents submit and retrieve knowledge tiles. The channel layer is the fleet's primary knowledge-sharing infrastructure.

From Oracle1's 6-layer interconnection model:
> Channel — IRC-like rooms (PLATO) — Status: Live (1,485+ rooms)

Additionally, Oracle1's Matrix bridge (Conduwuit homeserver at port 6167) provides real-time IRC-like communication between agents, particularly Oracle1 ↔ Forgemaster coordination.

### PLATO Rooms
PLATO's room system is the knowledge backbone:
- Rooms are named domain contexts (e.g., "engine-diagnostics", "deck-safety")
- Tiles are knowledge units: question + answer + confidence + tags
- Agents submit tiles, query rooms, and retrieve knowledge
- Sub-10ms latency across all operations
- "PLATO-FIRST: file knowledge to PLATO, keep files lean"

## What We Borrowed
The **channel abstraction** for inter-room messaging:
- Named channels for different communication patterns
- Broadcast delivery to multiple subscribers
- Priority ordering of messages
- Reliable delivery with acknowledgment

Specific concepts adapted:
- **DirectChannel** → Oracle1's directed bottles (for-{agent}/)
- **BroadcastChannel** → Oracle1's fleet broadcasts (for-fleet/, for-any-vessel/)
- **PriorityChannel** → Oracle1's I2I message type hierarchy (some types are more urgent)
- **ReliableChannel** → Oracle1's bottle delivery (no guarantee, but our version adds ack/retry)
- **ChannelMux** → Oracle1's multiple communication paths (bottles, issues, PRs, HTTP)

## How Our Implementation Differs

| Aspect | Oracle1's Channel/PLATO | Our ternary-channel |
|---|---|---|
| **Transport** | HTTP + git commits | In-memory Rust channels |
| **Persistence** | PostgreSQL + git history | Non-persistent (library, not service) |
| **Message types** | 20 I2I types across 6 categories | Generic `Message` with `TernaryPriority` |
| **Delivery guarantee** | None (best-effort git) | Optional via `ReliableChannel` (ack + retry) |
| **Broadcast** | Git directories (for-fleet/) | `BroadcastChannel` with subscriber management |
| **Priority** | I2I type prefix convention | `TernaryPriority` (Positive > Neutral > Negative) |
| **Multiplexing** | Multiple transport paths | `ChannelMux` with named sub-channels |
| **Ternary** | Not ternary-aware | Priority is ternary (Positive/Neutral/Negative) |

### Key Innovation: Ternary Priority Ordering
Our `PriorityChannel` dequeues in ternary order: Positive first, then Neutral, then Negative. Oracle1 has no priority queue — bottles are processed in whatever order beachcomb finds them. We add urgency semantics to every message.

### Key Innovation: Reliable Delivery
Oracle1's bottle protocol is explicitly best-effort: "No delivery guarantee. If urgent, follow up via other channels." Our `ReliableChannel` adds acknowledgment tracking and automatic retry with configurable max retries. Messages enter states: Pending → Acknowledged or Failed.

### Key Innovation: Channel Multiplexing
Our `ChannelMux` multiplexes many named logical channels over one connection. Oracle1 uses different mechanisms for different channels (bottles, issues, PRs, HTTP, Matrix). We unify these into a single abstraction with named sub-channels.

## See Also
- Oracle1 Architecture Review: `construct-coordination/notes/main/ORACLE1-ARCHITECTURE-REVIEW.md`
- Oracle1-Ternary Bridge: `construct-coordination/notes/main/ORACLE1-TERNARY-BRIDGE.md`
- PLATO room server: Oracle1's oracle1-box Docker (plato-room service on :8847)
