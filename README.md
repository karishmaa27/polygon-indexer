Real-time Polygon Blockchain Data Indexer
1. Introduction

This project implements a real-time blockchain data indexing system for the Polygon Network, focusing on the POL token.
The primary goal is to capture raw on-chain data and calculate cumulative net-flows of POL tokens to the Binance exchange.

The system is designed to be extensible so additional exchanges can be supported in the future.

2. Features

Real-time connection to the Polygon blockchain via RPC.

Detection of POL token transfers.

Tracking of cumulative net-flows to Binance addresses.

SQLite database integration for storage of raw transaction data and processed metrics.

Query interface (CLI / HTTP API) for retrieving results.

Modular design for easy scalability to multiple exchanges.

Focus on real-time indexing (no historical backfill in this phase).

3. Key Metric

Cumulative Net-Flows to Binance

Net Flow
=
POL Inflows (to Binance)
−
POL Outflows (from Binance)
Net Flow=POL Inflows (to Binance)−POL Outflows (from Binance)
4. Architecture

Blockchain: Polygon

Token: POL

Database: SQLite (can be swapped for other DBs in future)

Programming Language: Rust

Dependencies:

tokio → async runtime

web3 → Polygon RPC client

serde / serde_json → JSON handling

rusqlite → SQLite integration

warp (optional) → simple HTTP API

5. Binance Exchange Addresses

The following addresses are tracked for Binance inflows/outflows:

0xF977814e90dA44bFA03b6295A0616a897441aceC

0xe7804c37c13166fF0b37F5aE0BB07A3aEbb6e245

0x505e71695E9bc45943c58adEC1650577BcA68fD9

0x290275e3db66394C52272398959845170E4DCb88

0xD5C08681719445A5Fdce2Bda98b341A49050d821

0x082489A616aB4D46d1947eE3F912e080815b08DA

6. Database Schema

Transactions Table

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_number BIGINT,
    tx_hash TEXT,
    from_address TEXT,
    to_address TEXT,
    value TEXT,
    timestamp BIGINT
);


Netflows Table

CREATE TABLE IF NOT EXISTS netflows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exchange TEXT,
    cumulative_netflow REAL,
    last_updated BIGINT
);

7. Running the Project
Prerequisites

Rust (installed via rustup
)

Polygon RPC endpoint (Alchemy, Infura, or public RPC)

Steps
# Clone the repository
git clone https://github.com/<your-username>/polygon-indexer.git
cd polygon-indexer

# Run the project
cargo run

8. Querying Data

SQLite (direct query):

SELECT * FROM netflows;


HTTP API (if enabled):

GET http://localhost:3030/netflow


Example Response:

{
  "exchange": "Binance",
  "cumulative_netflow": 12345.67,
  "last_updated": 1694888888
}

9. Scalability

The system is modular: additional exchange addresses can be added.

Database schema supports multi-exchange tracking.

Future improvements can include:

Historical backfilling.

More tokens.

Alternative databases (Postgres, MySQL).

Dashboard/visualization layer.

10. Deliverables

Database schema.

Indexing logic in Rust.

Data transformation flow.

Query interface.

Documentation (this README).