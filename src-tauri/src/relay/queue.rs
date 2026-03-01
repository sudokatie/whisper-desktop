//! Message queue for offline handling.
//!
//! Queues outgoing messages when offline and flushes on reconnect.

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::sleep;

use super::client::{ConnectionState, RelayClient, RelayError};

/// Queued message for later sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub recipient: [u8; 32],
    pub payload: Vec<u8>,
    pub queued_at: u64,
    pub attempts: u32,
}

/// Queue errors
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Send failed: {0}")]
    Send(#[from] RelayError),
    #[error("Max retries exceeded")]
    MaxRetries,
}

/// Persistent message queue
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    path: Option<PathBuf>,
    max_retries: u32,
}

impl MessageQueue {
    /// Create a new in-memory queue
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            path: None,
            max_retries: 5,
        }
    }

    /// Create a persistent queue at the given path
    pub fn persistent(path: PathBuf) -> Result<Self, QueueError> {
        let queue = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str(&data)?
        } else {
            VecDeque::new()
        };

        Ok(Self {
            queue: Arc::new(Mutex::new(queue)),
            path: Some(path),
            max_retries: 5,
        })
    }

    /// Set max retry attempts
    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Queue a message for sending
    pub async fn enqueue(&self, recipient: [u8; 32], payload: Vec<u8>) -> Result<(), QueueError> {
        let msg = QueuedMessage {
            recipient,
            payload,
            queued_at: chrono::Utc::now().timestamp() as u64,
            attempts: 0,
        };

        {
            let mut queue = self.queue.lock().await;
            queue.push_back(msg);
        }

        self.persist().await
    }

    /// Get the next message without removing it
    pub async fn peek(&self) -> Option<QueuedMessage> {
        self.queue.lock().await.front().cloned()
    }

    /// Remove the front message
    pub async fn dequeue(&self) -> Option<QueuedMessage> {
        let msg = self.queue.lock().await.pop_front();
        if msg.is_some() {
            let _ = self.persist().await;
        }
        msg
    }

    /// Get queue length
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    /// Flush all queued messages through the client
    pub async fn flush(&self, client: &RelayClient) -> Result<usize, QueueError> {
        let mut sent = 0;
        
        loop {
            // Check connection state
            if client.state().await != ConnectionState::Connected {
                break;
            }

            let msg = {
                let mut queue = self.queue.lock().await;
                queue.pop_front()
            };

            let Some(mut msg) = msg else {
                break;
            };

            // Try to send
            match client.send(msg.recipient, msg.payload.clone()).await {
                Ok(_) => {
                    sent += 1;
                    self.persist().await?;
                }
                Err(_) => {
                    msg.attempts += 1;
                    if msg.attempts >= self.max_retries {
                        // Drop the message after max retries
                        self.persist().await?;
                        continue;
                    }
                    // Put it back at the front
                    {
                        let mut queue = self.queue.lock().await;
                        queue.push_front(msg);
                    }
                    break;
                }
            }
        }

        Ok(sent)
    }

    /// Flush with retry and backoff
    pub async fn flush_with_retry(
        &self,
        client: &RelayClient,
        max_attempts: u32,
    ) -> Result<usize, QueueError> {
        let mut total_sent = 0;
        let mut attempt = 0;

        while !self.is_empty().await && attempt < max_attempts {
            if client.state().await != ConnectionState::Connected {
                // Wait for reconnection
                let delay = Duration::from_secs(2u64.pow(attempt.min(5)));
                sleep(delay).await;
                attempt += 1;
                continue;
            }

            match self.flush(client).await {
                Ok(sent) => {
                    total_sent += sent;
                    if self.is_empty().await {
                        break;
                    }
                }
                Err(_) => {
                    attempt += 1;
                }
            }
        }

        Ok(total_sent)
    }

    /// Persist queue to disk if configured
    async fn persist(&self) -> Result<(), QueueError> {
        if let Some(path) = &self.path {
            let queue = self.queue.lock().await;
            let data = serde_json::to_string(&*queue)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    /// Clear all queued messages
    pub async fn clear(&self) -> Result<(), QueueError> {
        {
            let mut queue = self.queue.lock().await;
            queue.clear();
        }
        self.persist().await
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = MessageQueue::new();
        
        queue.enqueue([1u8; 32], vec![1, 2, 3]).await.unwrap();
        queue.enqueue([2u8; 32], vec![4, 5, 6]).await.unwrap();
        
        assert_eq!(queue.len().await, 2);
        
        let msg = queue.dequeue().await.unwrap();
        assert_eq!(msg.recipient, [1u8; 32]);
        assert_eq!(msg.payload, vec![1, 2, 3]);
        
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_persistent_queue() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("queue.json");
        
        // Create and populate queue
        {
            let queue = MessageQueue::persistent(path.clone()).unwrap();
            queue.enqueue([1u8; 32], vec![1, 2, 3]).await.unwrap();
            queue.enqueue([2u8; 32], vec![4, 5, 6]).await.unwrap();
        }
        
        // Reload and verify
        {
            let queue = MessageQueue::persistent(path).unwrap();
            assert_eq!(queue.len().await, 2);
            
            let msg = queue.peek().await.unwrap();
            assert_eq!(msg.recipient, [1u8; 32]);
        }
    }

    #[tokio::test]
    async fn test_queue_peek() {
        let queue = MessageQueue::new();
        
        assert!(queue.peek().await.is_none());
        
        queue.enqueue([1u8; 32], vec![1, 2, 3]).await.unwrap();
        
        let msg = queue.peek().await.unwrap();
        assert_eq!(msg.recipient, [1u8; 32]);
        
        // Peek doesn't remove
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_queue_clear() {
        let queue = MessageQueue::new();
        
        queue.enqueue([1u8; 32], vec![1, 2, 3]).await.unwrap();
        queue.enqueue([2u8; 32], vec![4, 5, 6]).await.unwrap();
        
        queue.clear().await.unwrap();
        
        assert!(queue.is_empty().await);
    }
}
