//! Message storage.
//!
//! Store and retrieve encrypted messages.

use crate::storage::database::Database;
use sqlx::Row;
use thiserror::Error;

/// Message storage errors.
#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Message not found")]
    NotFound,
}

/// Message delivery status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

impl MessageStatus {
    fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Pending => "pending",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
            MessageStatus::Failed => "failed",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "sent" => MessageStatus::Sent,
            "delivered" => MessageStatus::Delivered,
            "read" => MessageStatus::Read,
            "failed" => MessageStatus::Failed,
            _ => MessageStatus::Pending,
        }
    }
}

/// Message direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

impl MessageDirection {
    fn as_str(&self) -> &'static str {
        match self {
            MessageDirection::Incoming => "incoming",
            MessageDirection::Outgoing => "outgoing",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "outgoing" => MessageDirection::Outgoing,
            _ => MessageDirection::Incoming,
        }
    }
}

/// A stored message.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub peer_id: String,
    pub content: Vec<u8>, // Encrypted content
    pub timestamp: i64,
    pub status: MessageStatus,
    pub direction: MessageDirection,
}

/// Conversation summary.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub peer_id: String,
    pub last_message: Option<Vec<u8>>,
    pub last_message_at: Option<i64>,
    pub unread_count: i64,
}

/// Store a message.
pub async fn store_message(db: &Database, msg: &Message) -> Result<(), MessageError> {
    sqlx::query(
        "INSERT OR REPLACE INTO messages (id, peer_id, content, timestamp, status, direction) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&msg.id)
    .bind(&msg.peer_id)
    .bind(&msg.content)
    .bind(msg.timestamp)
    .bind(msg.status.as_str())
    .bind(msg.direction.as_str())
    .execute(db.pool())
    .await?;

    Ok(())
}

/// Get messages for a peer with pagination.
pub async fn get_messages(
    db: &Database,
    peer_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<Message>, MessageError> {
    let rows = sqlx::query(
        "SELECT id, peer_id, content, timestamp, status, direction FROM messages WHERE peer_id = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?",
    )
    .bind(peer_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Message {
            id: row.get("id"),
            peer_id: row.get("peer_id"),
            content: row.get("content"),
            timestamp: row.get("timestamp"),
            status: MessageStatus::from_str(row.get("status")),
            direction: MessageDirection::from_str(row.get("direction")),
        })
        .collect())
}

/// Get all conversations with last message and unread count.
pub async fn get_conversations(db: &Database) -> Result<Vec<Conversation>, MessageError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            m.peer_id,
            m.content as last_message,
            m.timestamp as last_message_at,
            (SELECT COUNT(*) FROM messages WHERE peer_id = m.peer_id AND status != 'read' AND direction = 'incoming') as unread_count
        FROM messages m
        WHERE m.timestamp = (SELECT MAX(timestamp) FROM messages WHERE peer_id = m.peer_id)
        ORDER BY m.timestamp DESC
        "#,
    )
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Conversation {
            peer_id: row.get("peer_id"),
            last_message: row.get("last_message"),
            last_message_at: row.get("last_message_at"),
            unread_count: row.get("unread_count"),
        })
        .collect())
}

/// Mark all messages from a peer as read.
pub async fn mark_read(db: &Database, peer_id: &str) -> Result<(), MessageError> {
    sqlx::query("UPDATE messages SET status = 'read' WHERE peer_id = ? AND direction = 'incoming'")
        .bind(peer_id)
        .execute(db.pool())
        .await?;

    Ok(())
}

/// Get total unread count.
pub async fn get_unread_count(db: &Database) -> Result<i64, MessageError> {
    let row = sqlx::query(
        "SELECT COUNT(*) as count FROM messages WHERE status != 'read' AND direction = 'incoming'",
    )
    .fetch_one(db.pool())
    .await?;

    Ok(row.get("count"))
}

/// Update message status.
pub async fn update_status(
    db: &Database,
    message_id: &str,
    status: MessageStatus,
) -> Result<(), MessageError> {
    let result = sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(message_id)
        .execute(db.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(MessageError::NotFound);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::contacts::{add_contact, Contact, TrustLevel};

    fn test_message(id: &str, peer_id: &str) -> Message {
        Message {
            id: id.to_string(),
            peer_id: peer_id.to_string(),
            content: vec![1, 2, 3],
            timestamp: 1000,
            status: MessageStatus::Pending,
            direction: MessageDirection::Outgoing,
        }
    }

    async fn create_test_contact(db: &Database, peer_id: &str) {
        let contact = Contact {
            peer_id: peer_id.to_string(),
            alias: "Test".to_string(),
            public_key: vec![1, 2, 3],
            trust_level: TrustLevel::Unknown,
            created_at: 1000,
            updated_at: 1000,
        };
        add_contact(db, &contact).await.unwrap();
    }

    #[tokio::test]
    async fn test_store_message() {
        let db = Database::open_memory().await.unwrap();
        create_test_contact(&db, "peer1").await;
        let msg = test_message("msg1", "peer1");
        
        store_message(&db, &msg).await.unwrap();
        
        let messages = get_messages(&db, "peer1", 10, 0).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg1");
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_get_messages_pagination() {
        let db = Database::open_memory().await.unwrap();
        create_test_contact(&db, "peer1").await;
        
        for i in 0..5 {
            let mut msg = test_message(&format!("msg{}", i), "peer1");
            msg.timestamp = 1000 + i;
            store_message(&db, &msg).await.unwrap();
        }
        
        let page1 = get_messages(&db, "peer1", 2, 0).await.unwrap();
        let page2 = get_messages(&db, "peer1", 2, 2).await.unwrap();
        
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
        // Ordered by timestamp DESC
        assert_eq!(page1[0].id, "msg4");
        assert_eq!(page1[1].id, "msg3");
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_get_conversations() {
        let db = Database::open_memory().await.unwrap();
        create_test_contact(&db, "peer1").await;
        create_test_contact(&db, "peer2").await;
        
        let mut msg1 = test_message("msg1", "peer1");
        msg1.timestamp = 1000;
        let mut msg2 = test_message("msg2", "peer2");
        msg2.timestamp = 2000;
        
        store_message(&db, &msg1).await.unwrap();
        store_message(&db, &msg2).await.unwrap();
        
        let convos = get_conversations(&db).await.unwrap();
        assert_eq!(convos.len(), 2);
        // Most recent first
        assert_eq!(convos[0].peer_id, "peer2");
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_mark_read() {
        let db = Database::open_memory().await.unwrap();
        create_test_contact(&db, "peer1").await;
        
        let mut msg = test_message("msg1", "peer1");
        msg.direction = MessageDirection::Incoming;
        msg.status = MessageStatus::Delivered;
        store_message(&db, &msg).await.unwrap();
        
        mark_read(&db, "peer1").await.unwrap();
        
        let messages = get_messages(&db, "peer1", 10, 0).await.unwrap();
        assert_eq!(messages[0].status, MessageStatus::Read);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_unread_count() {
        let db = Database::open_memory().await.unwrap();
        create_test_contact(&db, "peer1").await;
        
        for i in 0..3 {
            let mut msg = test_message(&format!("msg{}", i), "peer1");
            msg.direction = MessageDirection::Incoming;
            msg.status = MessageStatus::Delivered;
            store_message(&db, &msg).await.unwrap();
        }
        
        let count = get_unread_count(&db).await.unwrap();
        assert_eq!(count, 3);
        
        mark_read(&db, "peer1").await.unwrap();
        
        let count = get_unread_count(&db).await.unwrap();
        assert_eq!(count, 0);
        
        db.close().await;
    }
}
