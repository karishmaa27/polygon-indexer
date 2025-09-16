mod indexer;
mod db;
mod models;

use std::env;
use dotenv::dotenv;
use clap::{Parser, Subcommand};
use chrono::Utc;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// SQLite DB path
    #[arg(short, long, default_value = "pol_indexer.db")]
    db_path: String,

    /// Polygon RPC WebSocket URL (wss://...). If not provided, read from POLYGON_WS env.
    #[arg(short, long)]
    rpc: Option<String>,

    /// Run mode: index (start continuous indexer) or query (print latest netflow)
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index {
        /// Binance addresses comma separated (overrides default)
        #[arg(short, long)]
        binance: Option<String>,

        /// Token decimals (default 18)
        #[arg(short, long, default_value = "18")]
        decimals: u32,
    },
    Query {
        /// token address
        #[arg(short, long)]
        token: Option<String>,

        /// exchange name (default "Binance")
        #[arg(short, long, default_value = "Binance")]
        exchange: String,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    let db = db::open_db(&cli.db_path)?;
    match &cli.command {
        Commands::Index { binance, decimals } => {
            let rpc_url = cli.rpc.or_else(|| std::env::var("POLYGON_WS").ok())
                .expect("Provide RPC WebSocket URL via --rpc or POLYGON_WS env var (wss://...)");

            // Binance addresses provided by user if present; else use built-in list (from assignment)
            let binance_addrs: Vec<String> = match binance {
                Some(s) => s.split(',').map(|x| x.trim().to_string()).collect(),
                None => vec![
                    "0xF977814e90dA44bFA03b6295A0616a897441aceC".to_string(),
                    "0xe7804c37c13166fF0b37F5aE0BB07A3aEbb6e245".to_string(),
                    "0x505e71695E9bc45943c58adEC1650577BcA68fD9".to_string(),
                    "0x290275e3db66394C52272398959845170E4DCb88".to_string(),
                    "0xD5C08681719445A5Fdce2Bda98b341A49050d821".to_string(),
                    "0x082489A616aB4D46d1947eE3F912e080815b08DA".to_string(),
                ]
            };

            println!("Starting indexer at {} ...", rpc_url);
            indexer::run_indexer(&rpc_url, db, binance_addrs, *decimals).await?;
        },

        Commands::Query { token, exchange } => {
            // For Query mode: get latest cumulative_netflow record
            let tkn = token.unwrap_or_else(|| indexer::POL_CONTRACT.to_string());
            let mut stmt = db.prepare("SELECT cumulative_inflow_norm, cumulative_outflow_norm, cumulative_netflow_norm, as_of_block, updated_at
                FROM cumulative_netflow
                WHERE token_address = ?1 AND exchange_name = ?2
                ORDER BY id DESC LIMIT 1")?;
            let row = stmt.query_row(rusqlite::params![tkn, exchange], |r| {
                Ok((
                    r.get::<_, Option<f64>>(0)?,
                    r.get::<_, Option<f64>>(1)?,
                    r.get::<_, Option<f64>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                ))
            }).optional()?;

            if let Some((inflow, outflow, netflow, as_of_block, updated_at)) = row {
                println!("Token: {}\nExchange: {}\nAs of block: {:?}\nUpdated at: {:?}\nInflow: {:?}\nOutflow: {:?}\nNetflow: {:?}",
                    tkn, exchange, as_of_block, updated_at.map(|t| chrono::NaiveDateTime::from_timestamp(t, 0)), inflow, outflow, netflow);
            } else {
                println!("No cumulative record found for token {} and exchange {}", tkn, exchange);
            }
        }
    }

    Ok(())
}
