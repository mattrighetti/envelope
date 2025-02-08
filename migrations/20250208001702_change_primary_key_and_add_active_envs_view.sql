CREATE TABLE IF NOT EXISTS environments_new (
    env VARCHAR(50) NOT NULL,
    key TEXT NOT NULL,
    value TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY(env, key, value, created_at)
);

INSERT INTO environments_new
SELECT * FROM environments;

DROP TABLE environments;
ALTER TABLE environments_new RENAME TO environments;

-- latest_envs returns all the latest environments key-value pairs, including
-- the not active (null) values
CREATE VIEW IF NOT EXISTS latest_envs AS
SELECT
    env
    , key
    , value
    , created_at
FROM (
    SELECT
        *
        -- this will create an incremental value for each (env,key) pair with the same value.
        -- older value will have greater values, by picking rn = 1 sqlite will return the most
        -- recent value
        , ROW_NUMBER() OVER (PARTITION BY env, key ORDER BY created_at DESC) AS rn
    FROM environments
)
WHERE
    rn = 1;

-- active_envs returns all the active (non-null) latest environments key-value
-- pairs
CREATE VIEW IF NOT EXISTS active_envs AS
SELECT
    env
    , key
    , value
    , created_at
FROM (
    SELECT
        *
        -- this will create an incremental value for each (env,key) pair with the same value.
        -- older value will have greater values, by picking rn = 1 sqlite will return the most
        -- recent value
        , ROW_NUMBER() OVER (PARTITION BY env, key ORDER BY created_at DESC) AS rn
    FROM environments
)
WHERE
    rn = 1 AND
    value IS NOT NULL;
