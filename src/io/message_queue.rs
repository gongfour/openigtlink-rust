//! Message queue implementation for buffering and backpressure management
//!
//! Provides bounded and unbounded message queues for async message processing.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use crate::error::{IgtlError, Result};

/// Configuration for message queue behavior
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum number of messages in queue (None = unbounded)
    pub capacity: Option<usize>,
    /// Whether to drop oldest messages when queue is full (vs blocking)
    pub drop_on_full: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            capacity: Some(1000), // Default: bounded queue with 1000 messages
            drop_on_full: false,  // Default: block when full
        }
    }
}

impl QueueConfig {
    /// Create unbounded queue configuration
    pub fn unbounded() -> Self {
        Self {
            capacity: None,
            drop_on_full: false,
        }
    }

    /// Create bounded queue with specified capacity
    pub fn bounded(capacity: usize) -> Self {
        Self {
            capacity: Some(capacity),
            drop_on_full: false,
        }
    }

    /// Create bounded queue that drops oldest messages when full
    pub fn bounded_drop_old(capacity: usize) -> Self {
        Self {
            capacity: Some(capacity),
            drop_on_full: true,
        }
    }
}

/// Message queue for buffering raw message data
///
/// Supports both bounded and unbounded queues with optional message dropping.
pub struct MessageQueue {
    tx: mpsc::UnboundedSender<Vec<u8>>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
    config: QueueConfig,
    stats: Arc<Mutex<QueueStats>>,
}

/// Statistics for message queue
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// Total messages enqueued
    pub enqueued: u64,
    /// Total messages dequeued
    pub dequeued: u64,
    /// Total messages dropped (when queue full)
    pub dropped: u64,
    /// Current queue size
    pub current_size: usize,
    /// Peak queue size
    pub peak_size: usize,
}

impl MessageQueue {
    /// Create a new message queue with default configuration
    pub fn new() -> Self {
        Self::with_config(QueueConfig::default())
    }

    /// Create a new message queue with custom configuration
    pub fn with_config(config: QueueConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
            config,
            stats: Arc::new(Mutex::new(QueueStats::default())),
        }
    }

    /// Enqueue a message (non-blocking)
    ///
    /// # Arguments
    /// * `data` - Raw message bytes
    ///
    /// # Returns
    /// Ok(()) if enqueued, Err if queue is full and not configured to drop
    pub async fn enqueue(&self, data: Vec<u8>) -> Result<()> {
        let mut stats = self.stats.lock().await;

        // Check capacity if bounded
        if let Some(capacity) = self.config.capacity {
            if stats.current_size >= capacity {
                if self.config.drop_on_full {
                    // Drop the oldest message by dequeuing it
                    drop(stats); // Release lock before dequeue
                    if let Ok(_) = self.try_dequeue().await {
                        stats = self.stats.lock().await;
                        stats.dropped += 1;
                    } else {
                        return Err(IgtlError::Io(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Queue full and cannot drop oldest",
                        )));
                    }
                } else {
                    return Err(IgtlError::Io(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "Queue full",
                    )));
                }
            }
        }

        // Send message
        self.tx.send(data).map_err(|_| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Queue closed",
            ))
        })?;

        stats.enqueued += 1;
        stats.current_size += 1;
        if stats.current_size > stats.peak_size {
            stats.peak_size = stats.current_size;
        }

        Ok(())
    }

    /// Dequeue a message (blocking until message available)
    ///
    /// # Returns
    /// Message bytes or error if queue is closed
    pub async fn dequeue(&self) -> Result<Vec<u8>> {
        let mut rx = self.rx.lock().await;

        match rx.recv().await {
            Some(data) => {
                drop(rx); // Release lock before updating stats
                let mut stats = self.stats.lock().await;
                stats.dequeued += 1;
                stats.current_size = stats.current_size.saturating_sub(1);
                Ok(data)
            }
            None => Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Queue closed",
            ))),
        }
    }

    /// Try to dequeue a message (non-blocking)
    ///
    /// # Returns
    /// Some(data) if message available, None if queue is empty
    pub async fn try_dequeue(&self) -> Result<Vec<u8>> {
        let mut rx = self.rx.lock().await;

        match rx.try_recv() {
            Ok(data) => {
                drop(rx);
                let mut stats = self.stats.lock().await;
                stats.dequeued += 1;
                stats.current_size = stats.current_size.saturating_sub(1);
                Ok(data)
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                Err(IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "Queue empty",
                )))
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Queue closed",
                )))
            }
        }
    }

    /// Get current queue size
    pub async fn size(&self) -> usize {
        self.stats.lock().await.current_size
    }

    /// Get queue statistics
    pub async fn stats(&self) -> QueueStats {
        self.stats.lock().await.clone()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.stats.lock().await.current_size == 0
    }

    /// Get queue configuration
    pub fn config(&self) -> &QueueConfig {
        &self.config
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unbounded_queue() {
        let queue = MessageQueue::with_config(QueueConfig::unbounded());

        // Enqueue multiple messages
        for i in 0..100 {
            let data = vec![i as u8];
            queue.enqueue(data).await.unwrap();
        }

        assert_eq!(queue.size().await, 100);

        // Dequeue all messages
        for i in 0..100 {
            let data = queue.dequeue().await.unwrap();
            assert_eq!(data, vec![i as u8]);
        }

        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_bounded_queue() {
        let queue = MessageQueue::with_config(QueueConfig::bounded(10));

        // Fill queue
        for i in 0..10 {
            let data = vec![i as u8];
            queue.enqueue(data).await.unwrap();
        }

        // Try to enqueue when full (should fail)
        let result = queue.enqueue(vec![100]).await;
        assert!(result.is_err());

        // Dequeue one
        let _ = queue.dequeue().await.unwrap();

        // Now should succeed
        let result = queue.enqueue(vec![100]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bounded_drop_old() {
        let queue = MessageQueue::with_config(QueueConfig::bounded_drop_old(5));

        // Fill queue
        for i in 0..5 {
            let data = vec![i as u8];
            queue.enqueue(data).await.unwrap();
        }

        // Enqueue more (should drop oldest)
        for i in 5..10 {
            let data = vec![i as u8];
            queue.enqueue(data).await.unwrap();
        }

        // Queue should still be size 5
        assert_eq!(queue.size().await, 5);

        // First message should be 5 (0-4 were dropped)
        let data = queue.dequeue().await.unwrap();
        assert_eq!(data, vec![5]);

        // Check stats
        let stats = queue.stats().await;
        assert_eq!(stats.enqueued, 10);
        assert_eq!(stats.dropped, 5);
    }

    #[tokio::test]
    async fn test_try_dequeue_empty() {
        let queue = MessageQueue::new();

        let result = queue.try_dequeue().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let queue = MessageQueue::new();

        // Enqueue 10 messages
        for i in 0..10 {
            queue.enqueue(vec![i]).await.unwrap();
        }

        // Dequeue 5 messages
        for _ in 0..5 {
            let _ = queue.dequeue().await.unwrap();
        }

        let stats = queue.stats().await;
        assert_eq!(stats.enqueued, 10);
        assert_eq!(stats.dequeued, 5);
        assert_eq!(stats.current_size, 5);
        assert_eq!(stats.peak_size, 10);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let queue = Arc::new(MessageQueue::with_config(QueueConfig::bounded(100)));

        let queue_clone = queue.clone();
        let producer = tokio::spawn(async move {
            for i in 0..50 {
                queue_clone.enqueue(vec![i as u8]).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
            }
        });

        let queue_clone = queue.clone();
        let consumer = tokio::spawn(async move {
            for _ in 0..50 {
                let _ = queue_clone.dequeue().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
            }
        });

        producer.await.unwrap();
        consumer.await.unwrap();

        assert!(queue.is_empty().await);
    }
}
