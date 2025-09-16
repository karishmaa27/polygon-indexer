-- schema.sql

PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

-- Raw transactions table: stores each relevant ERC-20 Transfer log we process.
CREATE TABLE IF NOT EXISTS raw_transfers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_hash TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    token_address TEXT NOT NULL,
    from_addr TEXT NOT NULL,
    to_addr TEXT NOT NULL,
    amount_raw TEXT NOT NULL,       -- raw integer amount in token smallest units (string)
    amount_normalized REAL,         -- human-friendly value (amount_raw / 10^decimals)
    timestamp INTEGER NOT NULL,     -- block timestamp (unix seconds)
    processed_at INTEGER NOT NULL   -- when indexed (unix seconds)
);

CREATE INDEX IF NOT EXISTS idx_raw_transfers_token ON raw_transfers(token_address);
CREATE INDEX IF NOT EXISTS idx_raw_transfers_block ON raw_transfers(block_number);
CREATE UNIQUE INDEX IF NOT EXISTS uq_tx_log ON raw_transfers(tx_hash, log_index);

-- Cumulative net-flows computed per exchange (exchange_id), per token, cumulative over time
CREATE TABLE IF NOT EXISTS cumulative_netflow (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_address TEXT NOT NULL,
    exchange_name TEXT NOT NULL,
    as_of_block INTEGER NOT NULL,
    cumulative_inflow_raw TEXT NOT NULL,
    cumulative_outflow_raw TEXT NOT NULL,
    cumulative_netflow_raw TEXT NOT NULL,
    cumulative_inflow_norm REAL,
    cumulative_outflow_norm REAL,
    cumulative_netflow_norm REAL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_netflow_token_exchange ON cumulative_netflow(token_address, exchange_name);
