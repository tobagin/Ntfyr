ALTER TABLE subscription ADD COLUMN listen_since INTEGER NOT NULL DEFAULT 0;
UPDATE subscription
SET listen_since = read_until;
