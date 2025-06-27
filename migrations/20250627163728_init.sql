-- Initial migration for DC Bot SQLite database
-- This creates the foundational tables for message tracking and flush system

-- Messages table - records all user messages with full context
CREATE TABLE IF NOT EXISTS messages (
    message_id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    timestamp DATETIME NOT NULL
);

-- Indexes for efficient queries on messages table
CREATE INDEX IF NOT EXISTS idx_messages_user_guild ON messages(user_id, guild_id);
CREATE INDEX IF NOT EXISTS idx_messages_guild_channel ON messages(guild_id, channel_id);
CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);

-- Flush system table - tracks pending message flush votes
CREATE TABLE IF NOT EXISTS pending_flushes (
    message_id INTEGER PRIMARY KEY,
    notification_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    toilet_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    flusher_id INTEGER NOT NULL,
    threshold_count INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient queries on pending_flushes table
CREATE INDEX IF NOT EXISTS idx_pending_flushes_notification ON pending_flushes(notification_id);
CREATE INDEX IF NOT EXISTS idx_pending_flushes_created_at ON pending_flushes(created_at);