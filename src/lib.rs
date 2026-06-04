#![forbid(unsafe_code)]

//! Communication channel abstractions for inter-room messaging.
//!
//! Provides the comms backbone connecting rooms in a ternary fleet:
//! direct channels, broadcast, priority ordering, reliable delivery with
//! acknowledgment and retry, and multiplexing many logical channels over
//! one connection.

use std::collections::VecDeque;

// ---- Core types ----

/// Ternary priority for message ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TernaryPriority {
    /// Low priority, negative signal.
    Negative = -1,
    /// Normal priority, neutral.
    Neutral = 0,
    /// High priority, positive signal.
    Positive = 1,
}

/// A message payload with metadata.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub sender: String,
    pub recipient: String,
    pub payload: Vec<u8>,
    pub priority: TernaryPriority,
}

impl Message {
    pub fn new(id: u64, sender: &str, recipient: &str, payload: Vec<u8>, priority: TernaryPriority) -> Self {
        Self {
            id,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            payload,
            priority,
        }
    }
}

/// Channel state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    Open,
    Closed,
}

// ---- Channel trait ----

/// Core trait for all channel types.
pub trait Channel {
    /// Send a message through the channel.
    fn send(&mut self, msg: Message) -> Result<(), &'static str>;
    /// Try to receive a message (non-blocking).
    fn receive(&mut self) -> Option<Message>;
    /// Close the channel. No more sends or receives.
    fn close(&mut self);
    /// Check if the channel is open.
    fn is_open(&self) -> bool;
    /// Number of pending messages.
    fn pending_count(&self) -> usize;
}

// ---- DirectChannel ----

/// Point-to-point channel between two endpoints.
#[derive(Debug, Clone)]
pub struct DirectChannel {
    name: String,
    buffer: VecDeque<Message>,
    state: ChannelState,
    capacity: usize,
}

impl DirectChannel {
    pub fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            buffer: VecDeque::new(),
            state: ChannelState::Open,
            capacity,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Channel for DirectChannel {
    fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.state == ChannelState::Closed {
            return Err("Channel is closed");
        }
        if self.buffer.len() >= self.capacity {
            return Err("Channel buffer full");
        }
        self.buffer.push_back(msg);
        Ok(())
    }

    fn receive(&mut self) -> Option<Message> {
        self.buffer.pop_front()
    }

    fn close(&mut self) {
        self.state = ChannelState::Closed;
    }

    fn is_open(&self) -> bool {
        self.state == ChannelState::Open
    }

    fn pending_count(&self) -> usize {
        self.buffer.len()
    }
}

// ---- BroadcastChannel ----

/// One-to-many channel. Messages are delivered to all subscribers.
#[derive(Debug, Clone)]
pub struct BroadcastChannel {
    name: String,
    subscribers: Vec<String>,
    pending: VecDeque<Message>,
    state: ChannelState,
    capacity: usize,
}

impl BroadcastChannel {
    pub fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            subscribers: Vec::new(),
            pending: VecDeque::new(),
            state: ChannelState::Open,
            capacity,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Subscribe a recipient.
    pub fn subscribe(&mut self, recipient: &str) -> Result<(), &'static str> {
        if self.subscribers.contains(&recipient.to_string()) {
            return Err("Already subscribed");
        }
        self.subscribers.push(recipient.to_string());
        Ok(())
    }

    /// Unsubscribe a recipient.
    pub fn unsubscribe(&mut self, recipient: &str) {
        self.subscribers.retain(|s| s != recipient);
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn subscribers(&self) -> &[String] {
        &self.subscribers
    }
}

impl Channel for BroadcastChannel {
    fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.state == ChannelState::Closed {
            return Err("Channel is closed");
        }
        if self.pending.len() >= self.capacity {
            return Err("Channel buffer full");
        }
        self.pending.push_back(msg);
        Ok(())
    }

    fn receive(&mut self) -> Option<Message> {
        self.pending.pop_front()
    }

    fn close(&mut self) {
        self.state = ChannelState::Closed;
    }

    fn is_open(&self) -> bool {
        self.state == ChannelState::Open
    }

    fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

// ---- PriorityChannel ----

/// Messages ordered by ternary priority. Positive messages are dequeued first.
#[derive(Debug, Clone)]
pub struct PriorityChannel {
    name: String,
    positive: VecDeque<Message>,
    neutral: VecDeque<Message>,
    negative: VecDeque<Message>,
    state: ChannelState,
    capacity: usize,
}

impl PriorityChannel {
    pub fn new(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            positive: VecDeque::new(),
            neutral: VecDeque::new(),
            negative: VecDeque::new(),
            state: ChannelState::Open,
            capacity,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn total_pending(&self) -> usize {
        self.positive.len() + self.neutral.len() + self.negative.len()
    }
}

impl Channel for PriorityChannel {
    fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.state == ChannelState::Closed {
            return Err("Channel is closed");
        }
        if self.total_pending() >= self.capacity {
            return Err("Channel buffer full");
        }
        match msg.priority {
            TernaryPriority::Positive => self.positive.push_back(msg),
            TernaryPriority::Neutral => self.neutral.push_back(msg),
            TernaryPriority::Negative => self.negative.push_back(msg),
        }
        Ok(())
    }

    fn receive(&mut self) -> Option<Message> {
        if let Some(msg) = self.positive.pop_front() {
            return Some(msg);
        }
        if let Some(msg) = self.neutral.pop_front() {
            return Some(msg);
        }
        self.negative.pop_front()
    }

    fn close(&mut self) {
        self.state = ChannelState::Closed;
    }

    fn is_open(&self) -> bool {
        self.state == ChannelState::Open
    }

    fn pending_count(&self) -> usize {
        self.total_pending()
    }
}

// ---- ReliableChannel ----

/// Delivery status for reliable message tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Sent, awaiting acknowledgment.
    Pending,
    /// Acknowledged by recipient.
    Acknowledged,
    /// Failed after max retries.
    Failed,
}

/// A tracked message with retry state.
#[derive(Debug, Clone)]
struct TrackedMessage {
    msg: Message,
    status: DeliveryStatus,
    retries: u32,
    max_retries: u32,
}

/// Channel with acknowledgment and retry logic.
#[derive(Debug, Clone)]
pub struct ReliableChannel {
    name: String,
    inbox: VecDeque<Message>,
    outbox: Vec<TrackedMessage>,
    state: ChannelState,
    capacity: usize,
    default_max_retries: u32,
}

impl ReliableChannel {
    pub fn new(name: &str, capacity: usize, max_retries: u32) -> Self {
        Self {
            name: name.to_string(),
            inbox: VecDeque::new(),
            outbox: Vec::new(),
            state: ChannelState::Open,
            capacity,
            default_max_retries: max_retries,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Acknowledge receipt of a message by ID.
    pub fn ack(&mut self, msg_id: u64) -> Result<(), &'static str> {
        for tracked in &mut self.outbox {
            if tracked.msg.id == msg_id {
                tracked.status = DeliveryStatus::Acknowledged;
                return Ok(());
            }
        }
        Err("Message ID not found in outbox")
    }

    /// Retry all pending messages that haven't exceeded max retries.
    /// Returns number of messages retried.
    pub fn retry_pending(&mut self) -> u32 {
        let mut retried = 0u32;
        for tracked in &mut self.outbox {
            if tracked.status == DeliveryStatus::Pending && tracked.retries < tracked.max_retries {
                tracked.retries += 1;
                retried += 1;
            } else if tracked.status == DeliveryStatus::Pending
                && tracked.retries >= tracked.max_retries
            {
                tracked.status = DeliveryStatus::Failed;
            }
        }
        retried
    }

    /// Get delivery status for a message.
    pub fn status(&self, msg_id: u64) -> Option<DeliveryStatus> {
        self.outbox
            .iter()
            .find(|t| t.msg.id == msg_id)
            .map(|t| t.status)
    }

    /// Count messages with a given status.
    pub fn count_with_status(&self, status: DeliveryStatus) -> usize {
        self.outbox.iter().filter(|t| t.status == status).count()
    }

    /// Remove acknowledged and failed messages from outbox.
    pub fn clean_outbox(&mut self) {
        self.outbox
            .retain(|t| t.status == DeliveryStatus::Pending);
    }
}

impl Channel for ReliableChannel {
    fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.state == ChannelState::Closed {
            return Err("Channel is closed");
        }
        if self.inbox.len() + self.outbox.len() >= self.capacity {
            return Err("Channel buffer full");
        }
        self.outbox.push(TrackedMessage {
            msg,
            status: DeliveryStatus::Pending,
            retries: 0,
            max_retries: self.default_max_retries,
        });
        Ok(())
    }

    fn receive(&mut self) -> Option<Message> {
        self.inbox.pop_front()
    }

    fn close(&mut self) {
        self.state = ChannelState::Closed;
    }

    fn is_open(&self) -> bool {
        self.state == ChannelState::Open
    }

    fn pending_count(&self) -> usize {
        self.inbox.len() + self.outbox.len()
    }
}

// ---- ChannelMux ----

/// Multiplexes many named logical channels over one connection.
#[derive(Debug, Clone)]
pub struct ChannelMux {
    channels: Vec<(String, DirectChannel)>,
    state: ChannelState,
}

impl ChannelMux {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            state: ChannelState::Open,
        }
    }

    /// Add a named sub-channel.
    pub fn add_channel(&mut self, name: &str, capacity: usize) -> Result<(), &'static str> {
        if self.channels.iter().any(|(n, _)| n == name) {
            return Err("Channel name already exists");
        }
        self.channels
            .push((name.to_string(), DirectChannel::new(name, capacity)));
        Ok(())
    }

    /// Remove a named sub-channel.
    pub fn remove_channel(&mut self, name: &str) -> Result<(), &'static str> {
        let idx = self
            .channels
            .iter()
            .position(|(n, _)| n == name)
            .ok_or("Channel not found")?;
        self.channels.remove(idx);
        Ok(())
    }

    /// Send a message on a named sub-channel.
    pub fn send_on(&mut self, channel_name: &str, msg: Message) -> Result<(), &'static str> {
        if self.state == ChannelState::Closed {
            return Err("Mux is closed");
        }
        let ch = self
            .channels
            .iter_mut()
            .find(|(n, _)| n == channel_name)
            .ok_or("Channel not found")?;
        ch.1.send(msg)
    }

    /// Receive from a named sub-channel.
    pub fn receive_from(&mut self, channel_name: &str) -> Option<Message> {
        self.channels
            .iter_mut()
            .find(|(n, _)| n == channel_name)
            .and_then(|(_, ch)| ch.receive())
    }

    /// Receive from any sub-channel that has a message (round-robin).
    pub fn receive_any(&mut self) -> Option<(String, Message)> {
        for (name, ch) in &mut self.channels {
            if let Some(msg) = ch.receive() {
                return Some((name.clone(), msg));
            }
        }
        None
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Total pending across all sub-channels.
    pub fn total_pending(&self) -> usize {
        self.channels.iter().map(|(_, ch)| ch.pending_count()).sum()
    }

    pub fn channel_names(&self) -> Vec<&str> {
        self.channels.iter().map(|(n, _)| n.as_str()).collect()
    }

    pub fn is_open(&self) -> bool {
        self.state == ChannelState::Open
    }

    pub fn close(&mut self) {
        self.state = ChannelState::Closed;
        for (_, ch) in &mut self.channels {
            ch.close();
        }
    }
}

impl Default for ChannelMux {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(id: u64, sender: &str, recipient: &str, priority: TernaryPriority) -> Message {
        Message::new(id, sender, recipient, vec![1, 2, 3], priority)
    }

    // -- DirectChannel tests --

    #[test]
    fn direct_channel_send_receive() {
        let mut ch = DirectChannel::new("test", 10);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(ch.pending_count(), 1);
        let m = ch.receive().unwrap();
        assert_eq!(m.id, 1);
        assert_eq!(ch.pending_count(), 0);
    }

    #[test]
    fn direct_channel_fifo_order() {
        let mut ch = DirectChannel::new("test", 10);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(ch.receive().unwrap().id, 1);
        assert_eq!(ch.receive().unwrap().id, 2);
    }

    #[test]
    fn direct_channel_capacity() {
        let mut ch = DirectChannel::new("test", 2);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert!(ch.send(msg(3, "a", "b", TernaryPriority::Neutral)).is_err());
    }

    #[test]
    fn direct_channel_close() {
        let mut ch = DirectChannel::new("test", 10);
        assert!(ch.is_open());
        ch.close();
        assert!(!ch.is_open());
        assert!(ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).is_err());
    }

    #[test]
    fn direct_channel_empty_receive() {
        let mut ch = DirectChannel::new("test", 10);
        assert!(ch.receive().is_none());
    }

    // -- BroadcastChannel tests --

    #[test]
    fn broadcast_subscribe_unsubscribe() {
        let mut ch = BroadcastChannel::new("bc", 10);
        ch.subscribe("agent1").unwrap();
        ch.subscribe("agent2").unwrap();
        assert_eq!(ch.subscriber_count(), 2);
        ch.unsubscribe("agent1");
        assert_eq!(ch.subscriber_count(), 1);
    }

    #[test]
    fn broadcast_no_duplicate_subscriber() {
        let mut ch = BroadcastChannel::new("bc", 10);
        ch.subscribe("agent1").unwrap();
        assert!(ch.subscribe("agent1").is_err());
    }

    #[test]
    fn broadcast_send_receive() {
        let mut ch = BroadcastChannel::new("bc", 10);
        ch.subscribe("agent1").unwrap();
        ch.send(msg(1, "src", "all", TernaryPriority::Positive)).unwrap();
        let m = ch.receive().unwrap();
        assert_eq!(m.id, 1);
    }

    #[test]
    fn broadcast_close() {
        let mut ch = BroadcastChannel::new("bc", 10);
        ch.close();
        assert!(!ch.is_open());
        assert!(ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).is_err());
    }

    // -- PriorityChannel tests --

    #[test]
    fn priority_positive_first() {
        let mut ch = PriorityChannel::new("pq", 10);
        ch.send(msg(1, "a", "b", TernaryPriority::Negative)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.send(msg(3, "a", "b", TernaryPriority::Positive)).unwrap();
        assert_eq!(ch.receive().unwrap().id, 3); // positive first
        assert_eq!(ch.receive().unwrap().id, 2); // then neutral
        assert_eq!(ch.receive().unwrap().id, 1); // then negative
    }

    #[test]
    fn priority_neutral_before_negative() {
        let mut ch = PriorityChannel::new("pq", 10);
        ch.send(msg(1, "a", "b", TernaryPriority::Negative)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(ch.receive().unwrap().id, 2);
        assert_eq!(ch.receive().unwrap().id, 1);
    }

    #[test]
    fn priority_capacity() {
        let mut ch = PriorityChannel::new("pq", 2);
        ch.send(msg(1, "a", "b", TernaryPriority::Positive)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert!(ch.send(msg(3, "a", "b", TernaryPriority::Negative)).is_err());
    }

    #[test]
    fn priority_close_blocks_send() {
        let mut ch = PriorityChannel::new("pq", 10);
        ch.close();
        assert!(ch.send(msg(1, "a", "b", TernaryPriority::Positive)).is_err());
    }

    // -- ReliableChannel tests --

    #[test]
    fn reliable_send_and_ack() {
        let mut ch = ReliableChannel::new("rel", 10, 3);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(ch.status(1), Some(DeliveryStatus::Pending));
        ch.ack(1).unwrap();
        assert_eq!(ch.status(1), Some(DeliveryStatus::Acknowledged));
    }

    #[test]
    fn reliable_retry_and_fail() {
        let mut ch = ReliableChannel::new("rel", 10, 2);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.retry_pending(); // retry 1
        ch.retry_pending(); // retry 2 — max reached
        ch.retry_pending(); // this should mark as failed
        assert_eq!(ch.status(1), Some(DeliveryStatus::Failed));
    }

    #[test]
    fn reliable_clean_outbox() {
        let mut ch = ReliableChannel::new("rel", 10, 3);
        ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.send(msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        ch.ack(1).unwrap();
        ch.clean_outbox();
        assert_eq!(ch.count_with_status(DeliveryStatus::Pending), 1);
    }

    #[test]
    fn reliable_ack_unknown_fails() {
        let mut ch = ReliableChannel::new("rel", 10, 3);
        assert!(ch.ack(999).is_err());
    }

    #[test]
    fn reliable_close() {
        let mut ch = ReliableChannel::new("rel", 10, 3);
        ch.close();
        assert!(ch.send(msg(1, "a", "b", TernaryPriority::Neutral)).is_err());
    }

    // -- ChannelMux tests --

    #[test]
    fn mux_add_remove_channel() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        mux.add_channel("ch2", 10).unwrap();
        assert_eq!(mux.channel_count(), 2);
        mux.remove_channel("ch1").unwrap();
        assert_eq!(mux.channel_count(), 1);
    }

    #[test]
    fn mux_no_duplicate_names() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        assert!(mux.add_channel("ch1", 10).is_err());
    }

    #[test]
    fn mux_send_and_receive_named() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        mux.add_channel("ch2", 10).unwrap();
        mux.send_on("ch1", msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        mux.send_on("ch2", msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(mux.receive_from("ch1").unwrap().id, 1);
        assert_eq!(mux.receive_from("ch2").unwrap().id, 2);
    }

    #[test]
    fn mux_receive_any() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        mux.send_on("ch1", msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        let (name, m) = mux.receive_any().unwrap();
        assert_eq!(name, "ch1");
        assert_eq!(m.id, 1);
    }

    #[test]
    fn mux_close_all() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        mux.close();
        assert!(!mux.is_open());
        assert!(mux.send_on("ch1", msg(1, "a", "b", TernaryPriority::Neutral)).is_err());
    }

    #[test]
    fn mux_total_pending() {
        let mut mux = ChannelMux::new();
        mux.add_channel("ch1", 10).unwrap();
        mux.add_channel("ch2", 10).unwrap();
        mux.send_on("ch1", msg(1, "a", "b", TernaryPriority::Neutral)).unwrap();
        mux.send_on("ch2", msg(2, "a", "b", TernaryPriority::Neutral)).unwrap();
        assert_eq!(mux.total_pending(), 2);
    }

    #[test]
    fn mux_remove_nonexistent() {
        let mut mux = ChannelMux::new();
        assert!(mux.remove_channel("nope").is_err());
    }
}
