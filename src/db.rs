use rusqlite::{params, Connection, OptionalExtension};
use crate::models::TransferRecord;
use chrono::Utc;
use anyhow::Result;

/// Open or create SQLite DB at path
pub fn open_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // enable foreign keys etc
    conn.execute_batch(include_str!("../sql/schema.sql"))?;
    Ok(conn)
}

/// Insert a raw transfer record; ignore if duplicate (unique constraint)
pub fn insert_transfer(conn: &Connection, rec: &TransferRecord) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO raw_transfers
        (tx_hash, block_number, log_index, token_address, from_addr, to_addr, amount_raw, amount_normalized, timestamp, processed_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            rec.tx_hash,
            rec.block_number as i64,
            rec.log_index as i64,
            rec.token_address,
            rec.from_addr,
            rec.to_addr,
            rec.amount_raw,
            rec.amount_normalized,
            rec.timestamp,
            rec.processed_at,
        ],
    )?;
    Ok(())
}

/// Get current cumulative inflow/outflow for a token and exchange (uses Binance addresses list)
pub fn compute_and_store_cumulative(conn: &Connection, token_address: &str, exchange_name: &str, binance_addresses: &[&str], as_of_block: u64, decimals: u32) -> Result<()> {
    // Sum inflows (to any binance address) and outflows (from any binance address)
    let mut inflow_stmt = conn.prepare(&format!(
        "SELECT SUM(CAST(amount_raw AS INTEGER)) FROM raw_transfers WHERE token_address = ?1 AND to_addr IN ({})",
        binance_addresses.iter().map(|_| "?".to_string()).collect::<Vec<_>>().join(",")
    ))?;

    let mut outflow_stmt = conn.prepare(&format!(
        "SELECT SUM(CAST(amount_raw AS INTEGER)) FROM raw_transfers WHERE token_address = ?1 AND from_addr IN ({})",
        binance_addresses.iter().map(|_| "?".to_string()).collect::<Vec<_>>().join(",")
    ))?;

    // Build params vector: first param token_address, then all addresses
    let mut inflow_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    inflow_params.push(&token_address);
    for addr in binance_addresses { inflow_params.push(addr); }

    let mut outflow_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    outflow_params.push(&token_address);
    for addr in binance_addresses { outflow_params.push(addr); }

    let inflow_raw_opt: Option<i128> = inflow_stmt.query_row(inflow_params.as_slice(), |r| r.get(0)).optional()?.flatten();
    let outflow_raw_opt: Option<i128> = outflow_stmt.query_row(outflow_params.as_slice(), |r| r.get(0)).optional()?.flatten();

    let inflow_raw = inflow_raw_opt.unwrap_or(0);
    let outflow_raw = outflow_raw_opt.unwrap_or(0);
    let net_raw = inflow_raw - outflow_raw;

    // normalized
    let denom = 10f64.powi(decimals as i32);
    let inflow_norm = (inflow_raw as f64) / denom;
    let outflow_norm = (outflow_raw as f64) / denom;
    let net_norm = (net_raw as f64) / denom;

    // Insert a new row for as_of_block (could also UPSERT old row)
    conn.execute(
        "INSERT INTO cumulative_netflow
            (token_address, exchange_name, as_of_block, cumulative_inflow_raw, cumulative_outflow_raw, cumulative_netflow_raw, cumulative_inflow_norm, cumulative_outflow_norm, cumulative_netflow_norm, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            token_address,
            exchange_name,
            as_of_block as i64,
            inflow_raw.to_string(),
            outflow_raw.to_string(),
            net_raw.to_string(),
            inflow_norm,
            outflow_norm,
            net_norm,
            chrono::Utc::now().timestamp(),
        ],
    )?;

    Ok(())
}
