//! # Async Channel
//!
//! In this exercise, you will use `tokio::sync::mpsc` async channels to implement producer-consumer pattern.
//!
//! ## Concepts
//! - `tokio::sync::mpsc::channel` creates bounded async channels
//! - Async `send` and `recv`
//! - Channel closing mechanism (receiver returns None after all senders are dropped)

use tokio::sync::mpsc;

/// Async producer-consumer:
/// - Create a producer task that sends each element from items sequentially
/// - Create a consumer task that receives all elements and collects them into Vec for return
///
/// Hint: Set channel capacity to items.len().max(1)
pub async fn producer_consumer(items: Vec<String>) -> Vec<String> {
    let capacity = items.len().max(1);
    let (tx, mut rx) = mpsc::channel(capacity);

    // Producer: send every item, then drop `tx` so the channel closes.
    let producer = tokio::spawn(async move {
        for item in items {
            if tx.send(item).await.is_err() {
                // Receiver gone — nothing more we can do.
                break;
            }
        }
    });

    // Consumer: drain until all senders are dropped (recv returns None).
    let consumer = tokio::spawn(async move {
        let mut collected = Vec::new();
        while let Some(item) = rx.recv().await {
            collected.push(item);
        }
        collected
    });

    let _ = producer.await;
    consumer.await.unwrap()
}

/// Fan‑in pattern: multiple producers, one consumer.
/// Create `n_producers` producers, each sending `"producer {id}: message"`.
/// Consumer collects all messages, sorts them, and returns.
pub async fn fan_in(n_producers: usize) -> Vec<String> {
    let (tx, mut rx) = mpsc::channel(n_producers.max(1));

    // Each producer gets its own clone of the sender.
    for id in 0..n_producers {
        let tx = tx.clone();
        tokio::spawn(async move {
            let msg = format!("producer {id}: message");
            let _ = tx.send(msg).await;
        });
    }

    // Drop the original sender, otherwise the channel never closes.
    drop(tx);

    let mut collected = Vec::new();
    while let Some(msg) = rx.recv().await {
        collected.push(msg);
    }
    collected.sort();
    collected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_producer_consumer() {
        let items = vec!["hello".into(), "async".into(), "world".into()];
        let result = producer_consumer(items.clone()).await;
        assert_eq!(result, items);
    }

    #[tokio::test]
    async fn test_producer_consumer_empty() {
        let result = producer_consumer(vec![]).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_fan_in() {
        let result = fan_in(3).await;
        assert_eq!(
            result,
            vec![
                "producer 0: message",
                "producer 1: message",
                "producer 2: message",
            ]
        );
    }

    #[tokio::test]
    async fn test_fan_in_single() {
        let result = fan_in(1).await;
        assert_eq!(result, vec!["producer 0: message"]);
    }
}
