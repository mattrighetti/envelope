-- Add migration script here
CREATE TABLE IF NOT EXISTS environments(
env VARCHAR(50) NOT NULL,
key TEXT NOT NULL,
value TEXT,
created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
PRIMARY KEY(env,key,created_at)
);
