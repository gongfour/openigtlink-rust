//! Message Queue Demo
//!
//! Demonstrates message buffering and backpressure management using MessageQueue.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example message_queue_demo
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::message_queue::{MessageQueue, QueueConfig};
use openigtlink_rust::protocol::message::IgtlMessage;
use openigtlink_rust::protocol::types::{StatusMessage, TransformMessage};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Message Queue Demo ===\n");

    // Example 1: Unbounded queue
    println!("📦 Example 1: Unbounded Queue");
    {
        let queue = MessageQueue::with_config(QueueConfig::unbounded());

        // Enqueue many messages
        for i in 0..100 {
            let transform = TransformMessage::identity();
            let msg = IgtlMessage::new(transform, &format!("Device{}", i))?;
            let data = msg.encode()?;
            queue.enqueue(data).await?;
        }

        println!("  Enqueued: 100 messages");
        println!("  Queue size: {}", queue.size().await);

        // Dequeue 10 messages
        for _ in 0..10 {
            let _ = queue.dequeue().await?;
        }

        println!("  After dequeuing 10: {}", queue.size().await);

        let stats = queue.stats().await;
        println!("  Stats:");
        println!("    Enqueued: {}", stats.enqueued);
        println!("    Dequeued: {}", stats.dequeued);
        println!("    Peak size: {}", stats.peak_size);
    }

    println!();

    // Example 2: Bounded queue (blocking behavior)
    println!("📦 Example 2: Bounded Queue (Blocking)");
    {
        let queue = MessageQueue::with_config(QueueConfig::bounded(10));

        // Fill queue
        for i in 0..10 {
            let status = StatusMessage::ok(&format!("Message {}", i));
            let msg = IgtlMessage::new(status, "Device")?;
            let data = msg.encode()?;
            queue.enqueue(data).await?;
        }

        println!("  Queue full: {}/10", queue.size().await);

        // Try to enqueue when full
        let status = StatusMessage::ok("Extra message");
        let msg = IgtlMessage::new(status, "Device")?;
        let data = msg.encode()?;

        match queue.enqueue(data).await {
            Ok(_) => println!("  ✅ Enqueued (unexpected)"),
            Err(_) => println!("  ⚠️  Queue full - enqueue blocked (as expected)"),
        }

        // Dequeue one
        let _ = queue.dequeue().await?;
        println!("  Dequeued 1 message: {}/10", queue.size().await);

        // Now can enqueue
        let status = StatusMessage::ok("New message");
        let msg = IgtlMessage::new(status, "Device")?;
        let data = msg.encode()?;
        queue.enqueue(data).await?;
        println!("  ✅ Enqueued successfully: {}/10", queue.size().await);
    }

    println!();

    // Example 3: Bounded queue with drop-old behavior
    println!("📦 Example 3: Bounded Queue (Drop Oldest)");
    {
        let queue = MessageQueue::with_config(QueueConfig::bounded_drop_old(5));

        // Fill queue with messages 0-4
        for i in 0..5 {
            let status = StatusMessage::ok(&format!("Message {}", i));
            let msg = IgtlMessage::new(status, "Device")?;
            let data = msg.encode()?;
            queue.enqueue(data).await?;
        }

        println!("  Initial queue: 5/5");

        // Enqueue more messages (5-9) - should drop 0-4
        for i in 5..10 {
            let status = StatusMessage::ok(&format!("Message {}", i));
            let msg = IgtlMessage::new(status, "Device")?;
            let data = msg.encode()?;
            queue.enqueue(data).await?;
        }

        println!("  After enqueueing 5 more: {}/5", queue.size().await);

        let stats = queue.stats().await;
        println!("  Stats:");
        println!("    Enqueued: {}", stats.enqueued);
        println!("    Dropped: {} (oldest messages)", stats.dropped);
        println!("    Current: {}", stats.current_size);

        // Verify oldest messages were dropped
        let data = queue.dequeue().await?;
        let msg = IgtlMessage::<StatusMessage>::decode(&data)?;
        println!("  First message in queue: {:?}", msg.content);
    }

    println!();

    // Example 4: Producer-Consumer pattern
    println!("📦 Example 4: Producer-Consumer Pattern");
    {
        let queue = Arc::new(MessageQueue::with_config(QueueConfig::bounded(50)));

        // Producer task
        let producer_queue = queue.clone();
        let producer = tokio::spawn(async move {
            for i in 0..100 {
                let transform = TransformMessage::identity();
                let msg = IgtlMessage::new(transform, &format!("Tracker{}", i))
                    .expect("Failed to create message");
                let data = msg.encode().expect("Failed to encode");

                // Simulate slow producer
                tokio::time::sleep(Duration::from_millis(10)).await;

                match producer_queue.enqueue(data).await {
                    Ok(_) => {}
                    Err(_) => println!("    Producer: Queue full, waiting..."),
                }
            }
            println!("  Producer: Finished sending 100 messages");
        });

        // Consumer task (slower than producer)
        let consumer_queue = queue.clone();
        let consumer = tokio::spawn(async move {
            let mut count = 0;
            while count < 100 {
                // Simulate slow consumer
                tokio::time::sleep(Duration::from_millis(15)).await;

                match consumer_queue.try_dequeue().await {
                    Ok(_) => {
                        count += 1;
                        if count % 20 == 0 {
                            println!("  Consumer: Processed {} messages", count);
                        }
                    }
                    Err(_) => {
                        // Queue empty, wait a bit
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
            }
            println!("  Consumer: Finished processing 100 messages");
        });

        // Wait for both tasks
        producer.await.unwrap();
        consumer.await.unwrap();

        let stats = queue.stats().await;
        println!("  Final stats:");
        println!("    Enqueued: {}", stats.enqueued);
        println!("    Dequeued: {}", stats.dequeued);
        println!("    Peak size: {}", stats.peak_size);
    }

    println!();

    // Example 5: Multiple producers
    println!("📦 Example 5: Multiple Producers, Single Consumer");
    {
        let queue = Arc::new(MessageQueue::with_config(QueueConfig::bounded(100)));

        // Spawn 3 producers
        let mut producers = vec![];
        for producer_id in 0..3 {
            let queue_clone = queue.clone();
            let producer = tokio::spawn(async move {
                for i in 0..30 {
                    let status = StatusMessage::ok(&format!("P{}-M{}", producer_id, i));
                    let msg = IgtlMessage::new(status, &format!("Producer{}", producer_id))
                        .expect("Failed to create message");
                    let data = msg.encode().expect("Failed to encode");

                    queue_clone.enqueue(data).await.ok();
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            });
            producers.push(producer);
        }

        // Single consumer
        let consumer_queue = queue.clone();
        let consumer = tokio::spawn(async move {
            let mut count = 0;
            while count < 90 {
                #[allow(clippy::redundant_pattern_matching)]
                if let Ok(_) = consumer_queue.dequeue().await {
                    count += 1;
                }
            }
            count
        });

        // Wait for all producers
        for producer in producers {
            producer.await.unwrap();
        }

        let processed = consumer.await.unwrap();
        println!("  Producers: 3 x 30 messages = 90 total");
        println!("  Consumer processed: {} messages", processed);

        let stats = queue.stats().await;
        println!("  Queue stats:");
        println!("    Peak size: {}", stats.peak_size);
        println!("    Final size: {}", stats.current_size);
    }

    println!();

    // Example 6: Backpressure demonstration
    println!("📦 Example 6: Backpressure Management");
    {
        let queue = Arc::new(MessageQueue::with_config(QueueConfig::bounded(20)));

        // Fast producer
        let producer_queue = queue.clone();
        let producer = tokio::spawn(async move {
            let mut blocked_count = 0;
            for _i in 0..50 {
                let transform = TransformMessage::identity();
                let msg =
                    IgtlMessage::new(transform, "FastProducer").expect("Failed to create message");
                let data = msg.encode().expect("Failed to encode");

                loop {
                    match producer_queue.enqueue(data.clone()).await {
                        Ok(_) => break,
                        Err(_) => {
                            blocked_count += 1;
                            // Wait for consumer to catch up
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                }
            }
            blocked_count
        });

        // Slow consumer
        let consumer_queue = queue.clone();
        let consumer = tokio::spawn(async move {
            for _ in 0..50 {
                let _ = consumer_queue.dequeue().await;
                // Simulate slow processing
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });

        let blocked_count = producer.await.unwrap();
        consumer.await.unwrap();

        println!("  Fast producer blocked {} times", blocked_count);
        println!("  Backpressure successfully applied!");

        let stats = queue.stats().await;
        println!("  Peak queue size: {}/20", stats.peak_size);
    }

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 Key Takeaways:");
    println!("  - Unbounded queues: No size limit, risk of memory exhaustion");
    println!("  - Bounded queues: Fixed capacity, provides backpressure");
    println!("  - Drop-old mode: Maintains latest data, useful for real-time systems");
    println!("  - Producer-consumer: Decouples data production from processing");

    Ok(())
}
