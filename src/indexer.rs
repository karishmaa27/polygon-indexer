use ethers::providers::{Provider, Ws, Middleware, Http, StreamExt, FilterStream};
use ethers::types::{Filter, H256, Address, U256, Log};
use ethers::abi::ethereum_types::H160;
use anyhow::Result;
use std::sync::Arc;
use chrono::Utc;
use crate::models::TransferRecord;
use crate::db;
use rusqlite::Connection;

/// ERC20 Transfer topic signature
const TRANSFER_SIG: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a1c6f9f3d"; // keccak: Transfer(address,address,uint256)

/// POL token address on Polygon (mainnet)
/// Verified contract address for POL (Polygon Ecosystem Token):
/// 0x455e53CBB86018Ac2B8092FdCd39d8444aFFC3F6. See Polygonscan/Etherscan. (citation in README)
pub const POL_CONTRACT: &str = "0x455e53CBB86018Ac2B8092FdCd39d8444aFFC3F6";

/// Listen to new blocks and process logs for POL Transfer events
pub async fn run_indexer(ws_url: &str, conn: Connection, binance_addresses: Vec<String>, decimals: u32) -> Result<()> {
    let ws = Ws::connect(ws_url).await?;
    let provider = Provider::new(ws);
    let provider = Arc::new(provider);

    // We can subscribe to logs for the POL contract Transfer event
    let pol_addr: Address = POL_CONTRACT.parse()?;
    let transfer_topic: H256 = H256::from_slice(&hex::decode(&TRANSFER_SIG[2..])?);

    // Build a filter for logs where address == POL_CONTRACT and topic0 == Transfer
    let filter = Filter::new()
        .address(pol_addr)
        .topic0(transfer_topic);

    let mut stream = provider.subscribe_logs(&filter).await?;

    println!("Subscribed to POL Transfer logs. Waiting for new events...");

    while let Some(log) = stream.next().await {
        if let Err(e) = process_log(&provider, &conn, &log, &binance_addresses, decimals).await {
            eprintln!("Error processing log: {:?}", e);
        }
    }

    Ok(())
}

async fn process_log<M: Middleware + 'static>(provider: &Arc<M>, conn: &Connection, log: &Log, binance_addresses: &Vec<String>, decimals: u32) -> Result<()> {
    // log contains topics: [TransferSig, from, to] and data: amount (uint256)
    let tx_hash = log.transaction_hash
        .map(|h| format!("{:#x}", h))
        .unwrap_or_else(|| "<unknown_tx>".to_string());

    // Extract indexed params
    let from_raw = if log.topics.len() > 1 {
        let t = log.topics[1];
        // topics are 32-byte values with address right-padded; lower 20 bytes are address
        let addr = Address::from_slice(&t.as_bytes()[12..]);
        format!("{:?}", addr)
    } else { "<unknown>".to_string() };

    let to_raw = if log.topics.len() > 2 {
        let t = log.topics[2];
        let addr = Address::from_slice(&t.as_bytes()[12..]);
        format!("{:?}", addr)
    } else { "<unknown>".to_string() };

    // data holds the amount (uint256)
    let amount_u256 = U256::from_big_endian(&log.data.0);
    let amount_raw_str = amount_u256.to_string();

    // normalized
    let denom = 10u128.checked_pow(decimals).unwrap_or(1) as f64;
    let amount_norm = amount_u256.as_u128() as f64 / denom; // note: as_u128 may panic if > 128 bits

    // Get block timestamp (fetch block)
    let block_num = log.block_number.unwrap_or_default().as_u64();
    let block = provider.get_block(log.block_number.unwrap()).await?;
    let timestamp = block.and_then(|b| b.timestamp.as_u64().into()).map(|t:u64| t as i64).unwrap_or_else(|| Utc::now().timestamp());

    // create record
    let rec = TransferRecord {
        tx_hash: tx_hash.clone(),
        block_number: block_num,
        log_index: log.log_index.unwrap_or_default().as_u64(),
        token_address: format!("{:?}", log.address),
        from_addr: from_raw.clone(),
        to_addr: to_raw.clone(),
        amount_raw: amount_raw_str.clone(),
        amount_normalized: amount_norm,
        timestamp,
        processed_at: Utc::now().timestamp(),
    };

    db::insert_transfer(conn, &rec)?;

    // If this transfer involves Binance addresses, update the cumulative table
    let binance_addrs_lower: Vec<String> = binance_addresses.iter().map(|a| a.to_lowercase()).collect();
    if binance_addrs_lower.contains(&from_raw.to_lowercase()) || binance_addrs_lower.contains(&to_raw.to_lowercase()) {
        db::compute_and_store_cumulative(conn, POL_CONTRACT, "Binance", &binance_addrs_lower.iter().map(String::as_str).collect::<Vec<_>>(), block_num, decimals)?;
        println!("Processed transfer {}: {} -> {} amount {} (raw)", tx_hash, from_raw, to_raw, amount_raw_str);
    } else {
        // not involving Binance; store raw only
    }

    Ok(())
}
